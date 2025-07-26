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
            // 1NNN: Jumps to address NNN
            0x1000 => {
                let nnn = (opcode & 0x0FFF) as u16;
                self.pc = nnn;
                return Ok(());
            }
            _ => {
                println!("Opcode {:04X} not implemented yet", opcode);
            }
        }

        self.pc += 2;

        Ok(())
    }
}
