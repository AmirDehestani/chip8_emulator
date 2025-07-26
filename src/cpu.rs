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
}
