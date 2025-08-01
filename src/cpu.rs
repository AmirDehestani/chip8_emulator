use rand::Rng;
use crate::constants::{
    DISPLAY_WIDTH,
    DISPLAY_HEIGHT,
    INPUTS_COUNT,
    REGISTERS_COUNT,
    MEMORY_SIZE,
    STACK_SIZE,
    STARTING_MEMORY_ADDRESS,
    FONTSET_START_ADDRESS,
    FONTSET,
    BYTES_PER_FONT
};

pub struct CPU {
    pub v: [u8; REGISTERS_COUNT], // 16 8-bit general purpose registers named V0 to VF
    pub i: u16, // Address register
    pub pc: u16,
    pub memory: [u8; MEMORY_SIZE],
    pub stack: [u16; STACK_SIZE],
    pub sp: u8, // Stack pointer
    pub delay_timer: u8, // Both timer counts down from 60hz to 0
    pub sound_timer: u8,
    pub display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub input: [bool; INPUTS_COUNT]
}

impl CPU {

    pub fn new() -> Self {
        let mut cpu = CPU {
            v: [0; REGISTERS_COUNT],
            i: 0,
            pc: STARTING_MEMORY_ADDRESS as u16,
            memory: [0; MEMORY_SIZE],
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            input: [false; INPUTS_COUNT]
        };

        cpu.memory[FONTSET_START_ADDRESS..FONTSET_START_ADDRESS + FONTSET.len()].copy_from_slice(&FONTSET);

        cpu
    }

    /// Loads ROM into memory
    pub fn load_rom(&mut self, path: &str) -> Result<(), std::io::Error> {
        let rom = std::fs::read(path)?;

        if STARTING_MEMORY_ADDRESS + rom.len() > MEMORY_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "ROM too large"));
        }

        self.memory[STARTING_MEMORY_ADDRESS..(STARTING_MEMORY_ADDRESS + rom.len())].copy_from_slice(&rom);

        println!("Loaded {} bytes", rom.len());

        Ok(())
    }

    /// Executes one CPU cycle
    pub fn tick(&mut self) -> Result<(), std::io::Error> {
        let opcode: u16 = self.fetch()?;
        // println!("Running opcode: {:x}", opcode);
        self.decode_and_execute(opcode)
    }

    /// Fetches the next 2-byte opcode from memory at the current program counter
    pub fn fetch(&self) -> Result<u16, std::io::Error> {
        if self.pc_idx() + 1 >= MEMORY_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Out of bounds"));
        }

        let opcode_high = self.memory[self.pc_idx()];
        let opcode_low = self.memory[self.pc_idx() + 1];
        Ok((opcode_high as u16) << 8 | (opcode_low as u16))
    }

    /// Decodes the opcode and executes the corresponding instruction
    pub fn decode_and_execute(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        match opcode & 0xF000 {
            0x0000 => self.dispatch_0xxx(opcode),
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xnn(opcode),
            0x4000 => self.op_4xnn(opcode),
            0x5000 => self.op_5xy0(opcode),
            0x6000 => self.op_6xnn(opcode),
            0x7000 => self.op_7xnn(opcode),
            0x8000 => self.dispatch_8xxx(opcode),
            0x9000 => self.op_9xy0(opcode),
            0xA000 => self.op_annn(opcode),
            0xB000 => self.op_bnnn(opcode),
            0xC000 => self.op_cxnn(opcode),
            0xD000 => self.op_dxyn(opcode),
            0xE000 => self.dispatch_exxx(opcode),
            0xF000 => self.dispatch_fxxx(opcode),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown opcode {:04X}", opcode)))
        }
    }

    /// Update the delay and sound timer
    pub fn update_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
        if self.sound_timer > 0 {
            println!("BEEP!");
        }
    }

    /// Dispatcher for 0-prefixed opcodes (e.g. 0XXX)
    fn dispatch_0xxx(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        match opcode {
            0x00E0 => self.op_00e0(),
            0x00EE => self.op_00ee(),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown opcode {:04X}", opcode)))
        }
    }

    /// Dispatcher for 8-prefixed opcodes (e.g. 8XXX)
    fn dispatch_8xxx(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        match opcode & 0xF00F {
            0x8000 => self.op_8xy0(opcode),
            0x8001 => self.op_8xy1(opcode),
            0x8002 => self.op_8xy2(opcode),
            0x8003 => self.op_8xy3(opcode),
            0x8004 => self.op_8xy4(opcode),
            0x8005 => self.op_8xy5(opcode),
            0x8006 => self.op_8xy6(opcode),
            0x8007 => self.op_8xy7(opcode),
            0x800E => self.op_8xye(opcode),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown opcode {:04X}", opcode)))
        }
    }

    /// Dispatcher for E-prefixed opcodes (e.g. EXXX)
    fn dispatch_exxx(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        match opcode & 0xF0FF{
            0xE09E => self.op_ex9e(opcode),
            0xE0A1 => self.op_exa1(opcode),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown opcode {:04X}", opcode)))
        }
    }

    /// Dispatcher for F-prefixed opcodes (e.g. FXXX)
    fn dispatch_fxxx(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        match opcode & 0xF0FF{
            0xF007 => self.op_fx07(opcode),
            0xF00A => self.op_fx0a(opcode),
            0xF015 => self.op_fx15(opcode),
            0xF018 => self.op_fx18(opcode),
            0xF01E => self.op_fx1e(opcode),
            0xF029 => self.op_fx29(opcode),
            0xF033 => self.op_fx33(opcode),
            0xF055 => self.op_fx55(opcode),
            0xF065 => self.op_fx65(opcode),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown opcode {:04X}", opcode)))
        }
    }

    /// 00E0: Clears the screen
    fn op_00e0(&mut self) -> Result<(), std::io::Error> {
        self.display.fill(0);
        self.pc += 2;
        Ok(())
    }

    /// 00EE: Returns from a subroutine
    fn op_00ee(&mut self) -> Result<(), std::io::Error> {
        if self.sp_idx() == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Stack underflow"));
        }

        self.sp -= 1;
        let return_addr = self.stack[self.sp_idx()];
        self.pc = return_addr;
        Ok(())
    }

    /// 1NNN: Jumps to address NNN
    fn op_1nnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let nnn = CPU::get_nnn(opcode);
        self.pc = nnn;
        Ok(())
    }

    /// 2NNN: Calls subroutine at NNN
    fn op_2nnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {        
        if self.sp_idx() >= STACK_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Stack overflow"));
        }

        let nnn = CPU::get_nnn(opcode);
        self.stack[self.sp_idx()] = self.pc + 2; // Save address of the next instruction
        self.sp += 1;
        self.pc = nnn;
        Ok(())
    }

    /// 3XNN: Skips the next instruction if VX equals NN
    /// Usually the next instruction is a jump to skip a code block
    fn op_3xnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let nn = CPU::get_nn(opcode);
        let vx = self.v[x];

        if vx == nn {
            self.pc += 4;
        } else {
            self.pc += 2;
        }

        Ok(())
    }

    /// 4XNN: Skips the next instruction if VX does not equal NN
    /// Usually the next instruction is a jump to skip a code block
    fn op_4xnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let nn = CPU::get_nn(opcode);
        let vx = self.v[x];

        if vx != nn {
            self.pc += 4;
        } else {
            self.pc += 2;
        }

        Ok(())
    }

    /// 5XY0: Skips the next instruction if VX equals VY
    /// Usually the next instruction is a jump to skip a code block
    fn op_5xy0(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);
        let vx = self.v[x];
        let vy = self.v[y];

        if vx == vy {
            self.pc += 4;
        } else {
            self.pc += 2;
        }

        Ok(())
    }

    /// 6XNN: Sets VX to NN
    fn op_6xnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let nn = CPU::get_nn(opcode);
        self.v[x] = nn;
        self.pc += 2;
        Ok(())
    }

    /// 7XNN: Adds NN to VX (carry flag is not changed)
    fn op_7xnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let nn = CPU::get_nn(opcode);
        self.v[x] = self.v[x].wrapping_add(nn);
        self.pc += 2;
        Ok(())
    }

    /// 8XY0: Sets VX to the value of VY
    fn op_8xy0(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vy = self.v[y];
        self.v[x] = vy;

        self.pc += 2;
        Ok(())
    }

    /// 8XY1: Sets VX to VX or VY (bitwise OR operation)
    fn op_8xy1(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vx | vy;

        self.pc += 2;
        Ok(())
    }

    /// 8XY2: Sets VX to VX and VY (bitwise AND operation)
    fn op_8xy2(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vx & vy;

        self.pc += 2;
        Ok(())
    }

    /// 8XY3: Sets VX to VX xor VY
    fn op_8xy3(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vx ^ vy;

        self.pc += 2;
        Ok(())
    }

    /// 8XY4: Adds VY to VX. VF is set to 1 when there's an overflow, and to 0 when there is not
    fn op_8xy4(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vx = self.v[x];
        let vy = self.v[y];
        
        let (sum, did_overflow) = vx.overflowing_add(vy);
        self.v[x] = sum;
        self.v[0xF] = if did_overflow { 1 } else { 0 };

        self.pc += 2;
        Ok(())
    }

    /// 8XY5: VY is subtracted from VX. VF is set to 0 when there's an underflow, and 1 when there is not
    /// (i.e. VF set to 1 if VX >= VY and 0 if not)
    fn op_8xy5(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vx = self.v[x];
        let vy = self.v[y];
        
        let (result, did_underflow) = vx.overflowing_sub(vy);
        self.v[x] = result;
        self.v[0xF] = if did_underflow { 0 } else { 1 };

        self.pc += 2;
        Ok(())
    }

    /// 8XY6: Shifts VX to the right by 1, then stores the least significant bit of VX prior to the shift into VF
    fn op_8xy6(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);

        let vx = self.v[x];
        let vx_lsb = vx & 0x01;

        self.v[x] = vx >> 1;
        self.v[0xF] = vx_lsb;

        self.pc += 2;
        Ok(())
    }

    /// 8XY7: Sets VX to VY minus VX. VF is set to 0 when there's an underflow, and 1 when there is not
    /// (i.e. VF set to 1 if VY >= VX)
    fn op_8xy7(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);

        let vx = self.v[x];
        let vy = self.v[y];
        
        let (result, did_underflow) = vy.overflowing_sub(vx);
        self.v[x] = result;
        self.v[0xF] = if did_underflow { 0 } else { 1 };

        self.pc += 2;
        Ok(())
    }

    /// 8XYE: Shifts VX to the left by 1, then sets VF to 1 if the most significant bit 
    /// of VX prior to that shift was set, or to 0 if it was unset
    fn op_8xye(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);

        let vx = self.v[x];
        let vx_msb = (vx >> 7) & 0x01;

        self.v[x] = vx << 1;
        self.v[0xF] = vx_msb;

        self.pc += 2;
        Ok(())
    }

    /// 9XY0: Skips the next instruction if VX does not equal VY
    /// Usually the next instruction is a jump to skip a code block
    fn op_9xy0(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);
        let vx = self.v[x];
        let vy = self.v[y];

        if vx != vy {
            self.pc += 4;
        } else {
            self.pc += 2;
        }

        Ok(())
    }

    /// ANNN: Sets I to the address NNN
    fn op_annn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let nnn = CPU::get_nnn(opcode);
        self.i = nnn;
        self.pc += 2;
        Ok(())
    }

    /// BNNN: Jumps to the address NNN plus V0
    fn op_bnnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let nnn = CPU::get_nnn(opcode);
        let v0 = self.v[0x0] as u16;
        self.pc = nnn + v0;
        Ok(())
    }

    /// CXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN
    fn op_cxnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let nn = CPU::get_nn(opcode);
        let mut rng = rand::thread_rng();
        
        self.v[x] = nn & rng.gen_range(0..=255);

        self.pc += 2;
        Ok(())
    }

    /// DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels
    /// Each row of 8 pixels is read as bit-coded starting from memory location I
    /// I value does not change after the execution of this instruction
    /// VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen.
    fn op_dxyn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let y = CPU::get_y(opcode);
        let n = (opcode & 0x00F) as usize;

        let row_offset = self.v[y] as usize;
        let col_offset = self.v[x] as usize;

        self.v[0xF] = 0; // Reset collision flag

        for row in 0..n {
            let sprite_byte = self.memory[self.i as usize + row];

            for col in 0..8 { // 8 pixels in each row
                let display_x = (col + col_offset) % DISPLAY_WIDTH;
                let display_y = (row + row_offset) % DISPLAY_HEIGHT;

                let current_pixel = self.display[display_y * DISPLAY_WIDTH + display_x];
                let pixel = ((sprite_byte >> (7 - col)) & 0x1) as u8;
                let new_pixel = current_pixel ^ pixel;
                self.display[display_y * DISPLAY_WIDTH + display_x] = new_pixel as u8;

                if current_pixel == 1 && pixel == 1 {
                    self.v[0xF] = 1;
                }
            }
        }

        self.pc += 2;
        Ok(())
    }

    /// EX9E: Skips the next instruction if the key stored in VX (only consider the lowest nibble) is pressed
    /// Usually the next instruction is a jump to skip a code block
    fn op_ex9e(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let key = (self.v[x] & 0x0F) as usize;
        if self.input[key] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        Ok(())
    }

    /// EXA1: Skips the next instruction if the key stored in VX (only consider the lowest nibble) is not pressed
    /// Usually the next instruction is a jump to skip a code block
    fn op_exa1(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let key = (self.v[x] & 0x0F) as usize;
        if !self.input[key] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        Ok(())
    }

    /// FX07: Sets VX to the value of the delay timer
    fn op_fx07(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        self.v[x] = self.delay_timer;
        self.pc += 2;
        Ok(())
    }

    /// FX0A: A key press is awaited, and then stored in VX
    /// Blocking operation, all instruction halted until next key event, delay and sound timers should continue processing.
    fn op_fx0a(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        for (key, pressed) in self.input.iter().enumerate() {
            if *pressed {
                self.v[x] = key as u8;
                self.pc += 2;
                return Ok(());
            }
        }
        // No key is pressed. The PC is not updated and the insteruction is repeated
        Ok(())
    }

    /// FX15: Sets the delay timer to VX
    fn op_fx15(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        self.delay_timer = self.v[x];
        self.pc += 2;
        Ok(())
    }

    /// FX18: Sets the sound timer to VX
    fn op_fx18(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        self.sound_timer = self.v[x];
        self.pc += 2;
        Ok(())
    }

    /// FX1E: Adds VX to I. VF is not affected
    fn op_fx1e(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        self.i += self.v[x] as u16;
        self.pc += 2;
        Ok(())
    }

    /// FX29: Sets I to the location of the sprite for the character in VX (only consider the lowest nibble).
    /// Characters 0-F (in hexadecimal) are represented by a 4x5 font
    fn op_fx29(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let character = self.v[x] as usize;

        if character > 0x0F {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid character in VX for FX29"));
        }

        self.i = (FONTSET_START_ADDRESS + (character * BYTES_PER_FONT)) as u16;
        self.pc += 2;
        Ok(())
    }

    /// FX33: Stores the binary-coded decimal representation of VX, with the hundreds digit in memory
    /// at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);
        let value = self.v[x];

        let hundreds = value / 100;
        let tens = (value / 10) % 10;
        let ones = value % 10;

        self.memory[self.i_idx()] = hundreds;
        self.memory[self.i_idx() + 1] = tens;
        self.memory[self.i_idx() + 2] = ones;

        self.pc += 2;
        Ok(())
    }

    /// FX55: Stores from V0 to VX (including VX) in memory, starting at address I
    /// The offset from I is increased by 1 for each value written, but I itself is left unmodified
    fn op_fx55(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);

        for i in 0..=x {
            self.memory[self.i_idx() + i] = self.v[i];
        }

        self.pc += 2;
        Ok(())
    }

    /// FX65: Fills from V0 to VX (including VX) with values from memory, starting at address I.
    /// The offset from I is increased by 1 for each value read, but I itself is left unmodified.
    fn op_fx65(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let x = CPU::get_x(opcode);

        for i in 0..=x {
            self.v[i] = self.memory[self.i_idx() + i]
        }

        self.pc += 2;
        Ok(())
    }

    /// Helper method to get program counter as usize
    fn pc_idx(&self) -> usize {
        self.pc as usize
    }

    /// Helper method to get stack pointer as usize
    fn sp_idx(&self) -> usize {
        self.sp as usize
    }

    /// Helper method to get the value of address register as usize
    fn i_idx(&self) -> usize {
        self.i as usize
    }

    /// Helper function to extract x from the opcode
    fn get_x(opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }

    /// Helper function to extract y from the opcode
    fn get_y(opcode: u16) -> usize {
        ((opcode & 0x00F0) >> 4) as usize
    }

    /// Helper function to extract nn from the opcode
    fn get_nn(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    /// Helper function to extract nnn from the opcode
    fn get_nnn(opcode: u16) -> u16 {
        (opcode & 0x0FFF) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_timers() {
        let mut cpu = CPU::new();
        cpu.delay_timer = 3;
        cpu.sound_timer = 2;

        cpu.update_timers();
        assert_eq!(cpu.delay_timer, 2);
        assert_eq!(cpu.sound_timer, 1);

        cpu.update_timers();
        assert_eq!(cpu.delay_timer, 1);
        assert_eq!(cpu.sound_timer, 0);

        cpu.update_timers();
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);

        // Should stay at 0
        cpu.update_timers();
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);
    }
}
