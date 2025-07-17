use crate::error::Error;
use crate::memory::MEM_SIZE;

pub struct ProgramCounter(u32);

impl ProgramCounter {
    pub fn new() -> Self {
        ProgramCounter(0)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn set(&mut self, addr: u32) {
        self.0 = addr
    }

    // Increments the program counter and returns
    // the pc before it was incremented (AKA i++).
    pub fn inc(&mut self) -> Result<u32, Error> {
        let pc = self.0;
        // All base instructions in RISC-V are 32 bits (4 bytes) long.
        // The pc tracks byte addresses, so each sequential instruction is plus 4 bytes.
        self.0 += 4;
        if pc > MEM_SIZE as u32 - 4 {
            return Err(Error::InvalidPC(pc, MEM_SIZE));
        }
        Ok(pc)
    }
}
