use crate::inst::*;

// RAM size
pub const MEM_SIZE: usize = 1024 * 128;

#[derive(Clone)]
pub enum Size {
    // 8 bit
    Byte = 1,
    // 16 bit
    HalfWord = 2,
    // 32 bit
    Word = 4,
}

impl From<LoadIInst> for Size {
    fn from(value: LoadIInst) -> Self {
        match value {
            LoadIInst::LB | LoadIInst::LBU => Size::Byte,
            LoadIInst::LH | LoadIInst::LHU => Size::HalfWord,
            LoadIInst::LW => Size::Word,
        }
    }
}

impl From<SInst> for Size {
    fn from(value: SInst) -> Self {
        match value {
            SInst::SB => Size::Byte,
            SInst::SH => Size::HalfWord,
            SInst::SW => Size::Word,
        }
    }
}

macro_rules! read_mem {
    ($ty:ty, $mem:expr, $from:expr, $to:expr) => {
        // Little-Endian (LE)
        // 0x12345678 is stored as:
        // Address: 0x1000 | 0x1001 | 0x1002 | 0x1003
        // Data:    0x78   | 0x56   | 0x34   | 0x12
        <$ty>::from_le_bytes($mem[$from as usize..$to as usize].try_into().unwrap()) as u32
    };
}

pub struct Memory([u8; MEM_SIZE]);

impl Memory {
    pub fn new() -> Self {
        Memory([0; MEM_SIZE])
    }

    pub fn read(&self, from: u32, size: Size, is_unsigned: bool) -> u32 {
        let to = from + size.clone() as u32;
        match (size, is_unsigned) {
            (Size::Byte, true) => read_mem!(u8, self.0, from, to),
            (Size::Byte, false) => read_mem!(i8, self.0, from, to),
            (Size::HalfWord, true) => read_mem!(u16, self.0, from, to),
            (Size::HalfWord, false) => read_mem!(i16, self.0, from, to),
            (Size::Word, _) => read_mem!(u32, self.0, from, to),
        }
    }

    pub fn write(&mut self, from: u32, size: Size, val: u32) {
        let slice = val.to_le_bytes();
        let from = from as usize;
        let len = size as usize;
        self.0[from..from + len].copy_from_slice(&slice[0..len])
    }

    // loads program to start of the memory
    pub fn load_program(&mut self, mut program: Vec<u8>) {
        program.resize_with(MEM_SIZE, || 0);
        self.0 = program.try_into().unwrap();
    }
}
