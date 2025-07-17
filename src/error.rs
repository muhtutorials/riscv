use crate::inst_format::{BFormat, IFormat, RFormat, SFormat};
use std::fmt::{Debug, Formatter};

pub enum Error {
    InvalidOpcode(usize),
    InvalidInstFormat(FormatError),
    InvalidPC(u32, usize),
    EndOfInstructions,
}

pub enum FormatError {
    R(RFormat),
    I(IFormat),
    S(SFormat),
    B(BFormat),
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InvalidOpcode(opcode) => format!("invalid opcode: {:07b}", opcode),
                Error::InvalidInstFormat(kind) => match kind {
                    FormatError::R(format) => format!(
                        "invalid R-format instruction: funct3: '{:03b}', funct7: '{:07b}'",
                        format.funct3, format.funct7
                    ),
                    FormatError::I(format) => format!(
                        "invalid I-format instruction: funct3: '{:03b}'",
                        format.funct3
                    ),
                    FormatError::S(format) => format!(
                        "invalid S-format instruction: funct3: '{:03b}'",
                        format.funct3
                    ),
                    FormatError::B(format) => format!(
                        "invalid B-format instruction: funct3: '{:03b}'",
                        format.funct3
                    ),
                },
                Error::InvalidPC(pc, mem_size) =>
                    format!("program counter (pc: {pc}) bigger than memory (mem_size: {mem_size}B)"),
                Error::EndOfInstructions =>
                    "program ran out of instructions! Use exit syscall to terminate gracefully."
                        .to_string(),
            }
        )
    }
}
