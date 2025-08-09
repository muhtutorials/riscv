use crate::memory::MEM_SIZE;

pub struct Registers([u32; 32]);

impl Registers {
    pub fn new() -> Self {
        let mut regs = Registers([0; 32]);
        // initializes stack pointer to the top of the stack
        // `x2` register is SP (stack pointer). Points to the top of the stack.
        regs.0[2] = MEM_SIZE as u32;
        regs
    }

    pub fn read(&self, reg: usize) -> u32 {
        assert!(reg < 32, "rvi32 has only 32 registers");
        // `x0` register in RISC-V is hardwired to 0
        if reg == 0 {
            0
        } else {
            self.0[reg]
        }
    }

    pub fn write(&mut self, reg: usize, val: u32) {
        assert!(reg < 32, "rvi32 has only 32 registers");
        if reg == 0 {
            return;
        }
        self.0[reg] = val;
    }
}
