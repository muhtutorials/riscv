use crate::memory::MEM_SIZE;

pub struct Registers([u32; 32]);

impl Registers {
    pub fn new() -> Self {
        let mut regs = Registers([0; 32]);
        // initializes stack pointer to top of stack
        // `x2` register is sp (stack pointer). Points to the top of the stack.
        regs.0[2] = MEM_SIZE as u32;
        regs
    }

    pub fn read(&self, i: usize) -> u32 {
        assert!(i < 32, "rvi32 has only 32 registers");
        // `x0` register in RISC-V is hardwired to 0
        if i == 0 {
            0
        } else {
            self.0[i]
        }
    }

    pub fn write(&mut self, i: usize, val: u32) {
        assert!(i < 32, "rvi32 has only 32 registers");
        if i == 0 {
            return;
        }
        self.0[i] = val;
    }
}
