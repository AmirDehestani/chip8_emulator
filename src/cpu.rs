// https://en.wikipedia.org/wiki/CHIP-8
const REGISTERS_COUNT: usize = 16;
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
// Programs start at memory address 0x200; first 512 bytes (0x000–0x1FF) are reserved for the interpreter in original CHIP-8
const STARTING_MEMORY_ADDRESS: usize = 0x200;

pub struct CPU {
    pub v: [u8; REGISTERS_COUNT], // 16 8-bit general purpose registers named V0 to VF
    pub i: u16, // Address register
    pub pc: u16,
    pub memory: [u8; MEMORY_SIZE],
    pub stack: [u16; STACK_SIZE],
    pub sp: u8, // Stack pointer
    pub delay_timer: u8, // Both timer counts down from 60hz to 0
    pub sound_timer: u8
}

impl CPU {

    pub fn new() -> Self {
        CPU {
            v: [0; REGISTERS_COUNT],
            i: 0,
            pc: STARTING_MEMORY_ADDRESS,
            memory: [0; MEMORY_SIZE],
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0
        }
    }

    pub fn reset(&mut self) {
        *self = CPU::new();
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), std::io::Error> {
        let rom = std::fs::read(path)?;

        if STARTING_MEMORY_ADDRESS + rom.len() > MEMORY_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "ROM too large"));
        }

        self.memory[STARTING_MEMORY_ADDRESS..(STARTING_MEMORY_ADDRESS + rom.len())].copy_from_slice(&rom);

        println!("Loaded {} bytes", rom.len());

        Ok(())
    }

    fn pc_idx(&self) -> usize {
        self.pc as usize
    }

    fn sp_idx(&self) -> usize {
        self.sp as usize
    }

    pub fn tick(&mut self) -> Result<(), std::io::Error>{
        if self.pc_idx() + 1 >= MEMORY_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Out of bounds"));
        }

        let opcode_high = self.memory[self.pc_idx()];
        let opcode_low = self.memory[self.pc_idx() + 1];
        let opcode: u16 = (opcode_high as u16) << 8 | (opcode_low as u16);

        println!("PC: {:03X} | Opcode: {:04X}", self.pc, opcode);

        match opcode & 0xF000 {
            0x1000 => return self.op_1nnn(opcode),
            0x6000 => return self.op_6xnn(opcode),
            0x7000 => return self.op_7xnn(opcode),
            0xA000 => return self.op_annn(opcode),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown opcode {:04X}", opcode)))
        }
    }

    /// 1NNN: Jumps to address NNN
    fn op_1nnn(&mut self, opcode: u16) -> Result<(), std::io::Error> {
        let nnn = (opcode & 0x0FFF) as u16;
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

    /// Helper function to extract x from the opcode
    fn get_x(opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
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
