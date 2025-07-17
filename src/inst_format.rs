// Extracts inclusive range of bits from integer.
// Can be sign- or zero-extended depending on n_type.
#[macro_export]
macro_rules! get_bits {
    // defaults to zero-extension
    ($n:expr, $from:expr, $to:expr) => {
        get_bits!($n, $from, $to, usize)
    };

    //   '''
    // 01101011, 3, 5
    ($n:expr, $from:expr, $to:expr, $n_type:ty) => {{
        // inclusive range
        // 3 = 5 - 3 + 1
        let range = $to - $from + 1;
        // Builds a binary number consisting of only 1s with the len of range.
        // So `3` becomes `111`.
        // (1 << 3) - 1
        // 00001000 - 1
        // 00000111
        let ones = (1 << range) - 1;
        // we only want to keep bits in the range
        // 111 << 3
        // 00111000
        let mask = ones << $from;
        // apply mask and move matched pattern to LSB
        // (01101011 & 00111000) >> 3
        // 00101000 >> 3
        // 00000101
        ($n as $n_type & mask) >> $from
    }};
}

// R-type (Register):
// Used for register-register ALU operations.
// It includes fields for opcode, funct3, funct7,
// destination register (rd), and two source registers (rs1, rs2).
//
// 31        25 24    20 19    15 14     12 11      7 6      0
// +-----------+--------+--------+---------+---------+-------+
// |  funct7   |  rs2   |  rs1   |  funct3 |   rd    | opcode |
// +-----------+--------+--------+---------+---------+-------+
//
// funct7 (7 bits): Additional function code (combined with funct3 to determine operation).
// rs2 (5 bits): Second source register operand.
// rs1 (5 bits): First source register operand.
// funct3 (3 bits): Function code (combined with opcode to determine operation).
// rd (5 bits): Destination register.
// opcode (7 bits): Operation code.
//
// Key Characteristics
//  - Register Operations: Performs operations using two source
//    registers (rs1 and rs2) and stores result in a destination register (rd).
//  - No Immediate Values: All operands come from registers.
//  - Major Arithmetic/Logical Ops: Used for most arithmetic and logical operations.
//  - Combined Function Fields: funct7 and funct3 together specify the exact operation.
//  - Consistent Field Placement: rs1, rs2, and rd fields are in the same
//    position as in other formats.
pub struct RFormat {
    pub rd: usize,
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub funct7: usize,
}

impl RFormat {
    pub fn new(raw_inst: u32) -> Self {
        Self {
            rd: get_bits!(raw_inst, 7, 11),
            funct3: get_bits!(raw_inst, 12, 14),
            rs1: get_bits!(raw_inst, 15, 19),
            rs2: get_bits!(raw_inst, 20, 24),
            funct7: get_bits!(raw_inst, 25, 31),
        }
    }
}

// I-type (Immediate):
// Used for immediate and load operations.
// It includes opcode, funct3, a 12-bit immediate value,
// a source register (rs1), and a destination register (rd).
//
// 31                20 19    15 14     12 11      7 6      0
// +-------------------+--------+---------+---------+-------+
// |     imm[11:0]     |  rs1   |  funct3 |   rd    | opcode |
// +-------------------+--------+---------+---------+-------+
//
// imm[11:0]: 12-bit immediate value (bits 31:20).
// rs1: 5-bit source register 1 (bits 19:15).
// funct3: 3-bit function code (bits 14:12).
// rd: 5-bit destination register (bits 11:7).
// opcode: 7-bit opcode (bits 6:0).
pub struct IFormat {
    pub rd: usize,
    pub funct3: usize,
    pub rs1: usize,
    pub imm: u32,
}

impl IFormat {
    pub fn new(raw_inst: u32) -> Self {
        Self {
            rd: get_bits!(raw_inst, 7, 11),
            funct3: get_bits!(raw_inst, 12, 14),
            rs1: get_bits!(raw_inst, 15, 19),
            // immediates are sign-extended!
            imm: get_bits!(raw_inst, 20, 31, i32) as u32,
        }
    }
}

// S-type (Store):
// Used for store instructions. It includes opcode, funct3,
// a 12-bit immediate value, a source register (rs1), and
// a second source register (rs2) which is also the base
// address for the store operation.
//
// 31        25 24    20 19    15 14     12 11      7 6      0
// +-----------+--------+--------+---------+---------+-------+
// | imm[11:5] |  rs2   |  rs1   |  funct3 | imm[4:0]| opcode |
// +-----------+--------+--------+---------+---------+-------+
//
// imm[11:5]: Upper 7 bits of 12-bit immediate (bits 31:25).
// rs2: 5-bit source register 2 containing data to store (bits 24:20).
// rs1: 5-bit source register 1 containing base address (bits 19:15).
// funct3: 3-bit function code specifying store type (bits 14:12).
// imm[4:0]: Lower 5 bits of 12-bit immediate (bits 11:7).
// opcode: 7-bit operation code (bits 6:0).
pub struct SFormat {
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
}

impl SFormat {
    pub fn new(raw_inst: u32) -> Self {
        let imm_lo = get_bits!(raw_inst, 7, 11, i32);
        let imm_hi = get_bits!(raw_inst, 25, 31, i32);
        // combines `imm_lo` and `imm_hi` into one integer
        let imm = ((imm_hi << 5) | imm_lo) as u32;
        Self {
            funct3: get_bits!(raw_inst, 12, 14),
            rs1: get_bits!(raw_inst, 15, 19),
            rs2: get_bits!(raw_inst, 20, 24),
            imm,
        }
    }
}

// B-type (Branch):
// Used for conditional branch instructions. It includes opcode,
// funct3, a 12-bit immediate value (offset), a source
// register (rs1), and a second source register (rs2).
//
// 31    30 29    25 24    20 19    15 14     12 11      8 7     6 5      0
// +-------+--------+--------+--------+---------+---------+-------+-------+
// |imm[12]|imm[10:5]|  rs2  |   rs1  |  funct3 |imm[4:1] |imm[11]| opcode|
// +-------+--------+--------+--------+---------+---------+-------+-------+
//
// imm[12]: Highest bit of 13-bit immediate (bit 31).
// imm[10:5]: Middle 6 bits of immediate (bits 30:25).
// rs2: 5-bit source register 2 (bits 24:20).
// rs1: 5-bit source register 1 (bits 19:15).
// funct3: 3-bit branch condition code (bits 14:12).
// imm[4:1]: Lower 4 bits of immediate (bits 11:8).
// imm[11]: Second-highest bit of immediate (bit 7).
// opcode: 6-bit operation code (bits 6:0).
pub struct BFormat {
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
}

impl BFormat {
    // RISC-V Spec: 2.3
    // The only difference between the S and B formats is that
    // the 12-bit immediate field is used to encode
    // branch offsets in multiples of 2 in the B format.
    // Instead of shifting all bits in the instruction-encoded
    // immediate left by one in hardware as is conventionally done,
    // the middle bits (imm[10:1]) and sign bit
    // stay in fixed positions, while the lowest bit in
    // S format (inst[7]) encodes a high-order bit in B format.
    pub fn new(raw_inst: u32) -> Self {
        let imm_11th_bit = get_bits!(raw_inst, 7, 7, i32);
        let imm_lo = get_bits!(raw_inst, 8, 11, i32);
        let imm_hi = get_bits!(raw_inst, 25, 30, i32);
        let imm_12th_bit = get_bits!(raw_inst, 31, 31, i32);
        let imm = (
            (imm_12th_bit << 12) | (imm_11th_bit << 11) | (imm_hi << 5) | (imm_lo << 1)
        ) as u32;
        Self {
            funct3: get_bits!(raw_inst, 12, 14),
            rs1: get_bits!(raw_inst, 15, 19),
            rs2: get_bits!(raw_inst, 20, 24),
            imm,
        }
    }
}

// J-type (Jump):
// Used for unconditional jump instructions.
// It includes opcode, a 20-bit immediate value
// (offset), and a destination register (rd).
//
// 31    30 29       21 20   20 19       12 11      7 6      0
// +-------+-----------+-------+-----------+---------+-------+
// |imm[20]| imm[10:1] |imm[11]| imm[19:12]|   rd    | opcode|
// +-------+-----------+-------+-----------+---------+-------+
//
// imm[20]: Highest bit of 21-bit immediate (bit 31).
// imm[10:1]: Middle 10 bits of immediate (bits 30:21).
// imm[11]: 11th bit of immediate (bit 20).
// imm[19:12]: Upper 8 bits of immediate (bits 19:12).
// rd: 5-bit destination register (bits 11:7).
// opcode: 7-bit operation code (bits 6:0).
pub struct JFormat {
    pub rd: usize,
    pub imm: u32,
}

impl JFormat {
    pub fn new(raw_inst: u32) -> Self {
        let imm_hi = get_bits!(raw_inst, 12, 19, i32);
        let imm_11th_bit = get_bits!(raw_inst, 20, 20, i32);
        let imm_lo = get_bits!(raw_inst, 21, 30, i32);
        let imm_20th_bit = get_bits!(raw_inst, 31, 31, i32);
        let imm = (
            (imm_20th_bit << 20) | (imm_hi << 12) | (imm_11th_bit << 11) | (imm_lo << 1)
        ) as u32;
        Self {
            rd: get_bits!(raw_inst, 7, 11),
            imm,
        }
    }
}

// U-type (Upper Immediate):
// Used for instructions that load a 20-bit immediate value into a register.
// It includes opcode, a 20-bit immediate value, and a destination register (rd).
//
// 31                             12 11      7 6      0
// +--------------------------------+---------+-------+
// |          imm[31:12]            |   rd    | opcode|
// +--------------------------------+---------+-------+
//
// imm[31:12]: 20-bit immediate value (bits 31:12).
// rd: 5-bit destination register (bits 11:7).
// opcode: 7-bit operation code (bits 6:0).
pub struct UFormat {
    pub rd: usize,
    pub imm: u32,
}

impl UFormat {
    pub fn new(raw_inst: u32) -> Self {
        Self {
            rd: get_bits!(raw_inst, 7, 11),
            imm: get_bits!(raw_inst, 12, 31, i32) as u32,
        }
    }
}
