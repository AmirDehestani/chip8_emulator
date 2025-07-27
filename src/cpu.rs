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

        cpu.memory[STARTING_MEMORY_ADDRESS..STARTING_MEMORY_ADDRESS + FONTSET.len()].copy_from_slice(&FONTSET);

        cpu
    }

    pub fn reset(&mut self) {
        *self = CPU::new();
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
            0x6000 => self.op_6xnn(opcode),
            0x7000 => self.op_7xnn(opcode),
            0xA000 => self.op_annn(opcode),
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
            0xF00A => self.op_fx0a(opcode),
            0xF029 => self.op_fx29(opcode),
            0xF033 => self.op_fx33(opcode),
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
        self.stack[self.sp_idx()] = self.pc;
        self.sp += 1;
        self.pc = nnn;
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

    /// ANNN: Sets I to the address NNN
    fn op_annn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let nnn = CPU::get_nnn(opcode);
        self.i = nnn;
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
