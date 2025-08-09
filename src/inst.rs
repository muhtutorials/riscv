// https://projectf.io/posts/riscv-cheat-sheet/
use crate::cpu::Cpu;
use crate::get_bits;
use crate::inst_format::*;
use crate::memory::{Memory, Size};
use std::ops::{BitAnd, BitOr, BitXor};

pub enum Inst {
    // register-register operations
    R(RInst, RFormat),
    // immediate operations
    I(IInst, IFormat),
    // store instructions
    S(SInst, SFormat),
    // branch instructions
    B(BInst, BFormat),
    // jump instructions
    J(JFormat),
    // upper immediate instructions
    U(UInst, UFormat),

    // This isn't an official instruction but just
    // so that the emulator doesn't crash on `ecall`.
    // Only handles exit for now, every other syscall is ignored.
    SysCall(SysCall),
}

pub enum SysCall {
    Exit(u8),
    Nop,
}

// 0x1F = 0b00011111 = 31.
// rs2 & 0x1F ensures only the least significant
// 5 bits of rs2 are used for shifting,
// because shifting a 32-bit value by ≥32 bits is
// meaningless (shifting by 32 would clear all bits).
pub enum RInst {
    // Addition
    // Format: ADD rd, rs1, rs2.
    // Operation: rd = rs1 + rs2.
    // Description: Adds two registers (with overflow ignored).
    ADD,
    // Subtraction
    // Format: SUB rd, rs1, rs2.
    // Operation: rd = rs1 - rs2.
    // Description: Subtracts rs2 from rs1.
    SUB,
    // Bitwise Exclusive OR
    // Format: XOR rd, rs1, rs2.
    // Operation: rd = rs1 ^ rs2.
    // Description: Performs a bitwise XOR.
    XOR,
    // Bitwise OR
    // Format: OR rd, rs1, rs2.
    // Operation: rd = rs1 | rs2.
    // Description: Performs a bitwise OR.
    OR,
    // Bitwise AND
    // Format: AND rd, rs1, rs2.
    // Operation: rd = rs1 & rs2.
    // Description: Performs a bitwise AND.
    AND,
    // Shift Left Logical
    // Format: SLL rd, rs1, rs2.
    // Operation: rd = rs1 << (rs2 & 0x1F) (for RV32I).
    // Description: Shifts rs1 left by rs2 bits (zeros fill the right).
    SLL,
    // Shift Right Logical
    // Format: SRL rd, rs1, rs2.
    // Operation: rd = rs1 >> (rs2 & 0x1F) (logical shift, zeros fill the left).
    // Description: Shifts rs1 right (unsigned, no sign extension).
    SRL,
    // Shift Right Arithmetic
    // Format: SRA rd, rs1, rs2.
    // Operation: rd = rs1 >> (rs2 & 0x1F) (arithmetic shift, sign-extended).
    // Description: Preserves the sign bit when shifting right (for signed numbers).
    SRA,
    // Set Less Than
    // Format: SLT rd, rs1, rs2.
    // Operation: rd = (rs1 < rs2) ? 1 : 0 (signed comparison).
    // Description: Sets rd to 1 if rs1 < rs2 (treating values as signed).
    SLT,
    // Set Less Than Unsigned
    // Format: SLTU rd, rs1, rs2.
    // Operation: rd = (rs1 < rs2) ? 1 : 0 (unsigned comparison).
    // Description: Sets rd to 1 if rs1 < rs2 (treating values as unsigned).
    SLTU,
}

impl RInst {
    fn op(self) -> impl FnOnce(u32, u32) -> u32 {
        match self {
            RInst::ADD => u32::wrapping_add,
            RInst::SUB => u32::wrapping_sub,
            RInst::XOR => u32::bitxor,
            RInst::OR => u32::bitor,
            RInst::AND => u32::bitand,
            RInst::SLL => |rs1, rs2| {
                // Takes first 5 bits.
                // See note at `RInst` declaration.
                let amount = get_bits!(rs2, 0, 4);
                rs1 << amount
            },
            RInst::SRL => |rs1, rs2| {
                let amount = get_bits!(rs2, 0, 4);
                rs1 >> amount
            },
            RInst::SRA => |rs1, rs2| {
                let amount = get_bits!(rs2, 0, 4, i32);
                (rs1 as i32 >> amount) as u32
            },
            RInst::SLT => |rs1, rs2| ((rs1 as i32) < (rs2 as i32)) as u32,
            RInst::SLTU => |rs1, rs2| (rs1 < rs2) as u32,
        }
    }
}

impl From<ArithIInst> for RInst {
    fn from(value: ArithIInst) -> Self {
        match value {
            ArithIInst::ADDI => RInst::ADD,
            ArithIInst::XORI => RInst::XOR,
            ArithIInst::ORI => RInst::OR,
            ArithIInst::ANDI => RInst::AND,
            ArithIInst::SLLI => RInst::SLL,
            ArithIInst::SRLI => RInst::SRL,
            ArithIInst::SRAI => RInst::SRA,
            ArithIInst::SLTI => RInst::SLT,
            ArithIInst::SLTIU => RInst::SLTU,
        }
    }
}

// the same as `RInst`, but instead of `rs2` `imm` is used.
// `I` at the end of an instruction stands for `immediate`.
pub enum ArithIInst {
    ADDI,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    SLTI,
    SLTIU,
}

pub enum LoadIInst {
    // Load Byte
    // Format: LB rd, offset (rs1).
    // Operation: Loads an 8-bit (1-byte) value from memory
    // at address rs1 + offset, sign-extends it to 32/64 bits,
    // and stores it in rd.
    LB,
    // Load Halfword
    // Format: LH rd, offset (rs1).
    // Operation: Loads a 16-bit (2-byte) value from memory
    // at address rs1 + offset, sign-extends it,
    // and stores it in rd.
    LH,
    // Load Word
    // Format: LW rd, offset (rs1).
    // Operation: Loads a 32-bit (4-byte) value from memory
    // at address rs1 + offset, sign-extends it (in RV64),
    // and stores it in rd.
    // In RV32, no extension is needed (32 bits fill the register).
    // In RV64, the 32-bit value is sign-extended to 64 bits.
    LW,
    // Load Byte Unsigned
    // Format: LBU rd, offset (rs1).
    // Operation: Loads an 8-bit (1-byte) value from memory
    // at address rs1 + offset, zero-extends it to 32/64 bits,
    // and stores it in rd.
    // Difference from LB:
    // LB sign-extends (preserves signed values).
    // LBU zero-extends (for unsigned values).
    LBU,
    // Load Halfword Unsigned
    // Format: LHU rd, offset (rs1).
    // Operation: Loads a 16-bit (2-byte) value from memory
    // at address rs1 + offset, zero-extends it,
    // and stores it in rd.
    // Difference from LH:
    // LH sign-extends (for signed values).
    // LHU zero-extends (for unsigned values).
    LHU,
}

impl LoadIInst {
    fn is_unsigned(&self) -> bool {
        matches!(self, LoadIInst::LBU | LoadIInst::LHU)
    }

    fn op(self, mem: &Memory) -> impl FnOnce(u32, u32) -> u32 + '_ {
        move |rs1, imm| {
            // TODO: why do we use an offset here?
            let from = u32::wrapping_add(rs1, imm);
            let is_unsigned = self.is_unsigned();
            let size = Size::from(self);
            mem.read(from, size, is_unsigned)
        }
    }
}

pub enum IInst {
    Arith(ArithIInst),
    Mem(LoadIInst),
    // Jump And Link Register
    // Jumping to an address stored in a register (indirect jumps).
    // Saving the return address (for function calls/returns).
    // jal  rd, imm       # rd = pc+4; pc += imm
    // jalr rd, rs1, imm  # rd = pc+4; pc = rs1+imm
    Jalr,
}

impl IInst {
    // TODO: why is the return type boxed?
    fn op(self, cpu: &mut Cpu) -> Box<dyn FnOnce(u32, u32) -> u32 + '_> {
        // Arithmetic operations are the same for R/I format,
        // only the second operand differs.
        match self {
            IInst::Arith(inst) => Box::new(RInst::from(inst).op()),
            IInst::Mem(inst) => Box::new(inst.op(&cpu.mem)),
            IInst::Jalr => Box::new(|rs1, imm| {
                let original_pc = cpu.pc.get();
                cpu.pc.set(u32::wrapping_add(rs1, imm));
                original_pc
            }),
        }
    }
}

// sw  # mem[rs1+imm] = rs2             ; store word
// sh  # mem[rs1+imm][0:15] = rs2[0:15] ; store half word
// sb  # mem[rs1+imm][0:7] = rs2[0:7]   ; store byte
pub enum SInst {
    // Store Byte
    SB,
    // Store Halfword
    SH,
    // Store Word
    SW,
}

impl SInst {
    fn op(self, mem: &mut Memory) -> impl FnOnce(u32, u32, u32) + '_ {
        move |rs1, rs2, imm| {
            let from = u32::wrapping_add(rs1, imm);
            let size = Size::from(self);
            mem.write(from, size, rs2)
        }
    }
}

// Branch Instructions:
// Inst  Full Name	                            Condition (Jump if...)  Type
// BEQ	 Branch if Equal	                    rs1 == rs2	            Signed
// BNE	 Branch if Not Equal	                rs1 != rs2	            Signed
// BLT	 Branch if Less Than	                rs1 < rs2 (signed)	    Signed
// BLTU	 Branch if Less Than (Unsigned)	        rs1 < rs2 (unsigned)	Unsigned
// BGE	 Branch if Greater or Equal	            rs1 >= rs2 (signed)	    Signed
// BGEU	 Branch if Greater or Equal (Unsigned)  rs1 >= rs2 (unsigned)   Unsigned
pub enum BInst {
    BEQ,
    BNE,
    BLT,
    BLTU,
    BGE,
    BGEU,
}

pub enum UInst {
    // Load Upper Immediate
    // Loads a 20-bit immediate value into the upper 20 bits
    // of a register, setting the lower 12 bits to zero.
    // Used to construct large constants or addresses
    // (e.g., for memory-mapped I/O or global variables).
    LUI,
    // Add Upper Immediate to PC
    // Adds a 20-bit immediate (shifted left by 12 bits) to the
    // current PC (Program Counter) and stores the result in a register.
    // Used for position-independent code
    // (e.g., accessing global data or functions relative to PC).
    AUIPC,
}

impl UInst {
    fn op(self, pc: u32) -> impl FnOnce(u32) -> u32 {
        // TODO: what does it do?
        move |imm| match self {
            UInst::LUI => imm << 12,
            UInst::AUIPC => u32::wrapping_add(pc - 4, imm << 12),
        }
    }
}

impl Inst {
    pub fn execute(self, cpu: &mut Cpu) {
        match self {
            Inst::R(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let rs2 = cpu.regs.read(format.rs2);
                // Arithmetic Logic Unit (ALU)
                let alu = inst.op();
                let result = alu(rs1, rs2);
                cpu.regs.write(format.rd, result)
            }
            Inst::I(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let alu = inst.op(cpu);
                let result = alu(rs1, format.imm);
                cpu.regs.write(format.rd, result);
            }
            Inst::S(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let rs2 = cpu.regs.read(format.rs2);
                let alu = inst.op(&mut cpu.mem);
                alu(rs1, rs2, format.imm);
            }
            Inst::B(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let rs2 = cpu.regs.read(format.rs2);
                let branch = match inst {
                    BInst::BEQ => rs1 == rs2,
                    BInst::BNE => rs1 != rs2,
                    BInst::BLT => (rs1 as i32) < (rs2 as i32),
                    BInst::BLTU => rs1 < rs2,
                    BInst::BGE => rs1 as i32 >= rs2 as i32,
                    BInst::BGEU => rs1 >= rs2,
                };
                // TODO: what does it do?
                if branch {
                    // The immediate value in a jump instruction
                    // is typically encoded as an offset relative
                    // to the current instruction's address (not the next one).
                    // Since the CPU has already incremented the PC by 4,
                    // you need to compensate by subtracting 4 to make the offset correct:
                    // jump = (current_pc + 4) + (offset - 4) = current_pc + offset
                    cpu.pc.set(u32::wrapping_add(
                        cpu.pc.get(),
                        u32::wrapping_sub(format.imm, 4),
                    ));
                }
            }
            Inst::J(format) => {
                cpu.regs.write(format.rd, cpu.pc.get());
                cpu.pc.set(u32::wrapping_add(
                    cpu.pc.get(),
                    u32::wrapping_sub(format.imm, 4),
                ));
            }
            Inst::U(inst, format) => {
                let alu = inst.op(cpu.pc.get());
                let result = alu(format.imm);
                cpu.regs.write(format.rd, result);
            }
            Inst::SysCall(..) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_byte() {
        let mut cpu = Cpu::new(false);
        cpu.regs.write(28, 12);
        // li t0, 42     # load the immediate 42 into register t0
        // li t6, 0x140  # load the immediate 0x140 (address) into register t6
        // sw t0, 0(t6)  # store the word in t0 to memory address in t6 with 0 byte offset
        // mem[0 + 3] = 12[0:7]
        let inst = Inst::S(
            SInst::SB,
            SFormat {
                funct3: 0x0,
                rs1: 0,
                rs2: 28,
                imm: 3,
            }
        );
        inst.execute(&mut cpu);
        assert_eq!(cpu.mem.read(3, Size::Byte, true), 12)
    }

    #[test]
    fn lui() {
        let mut cpu = Cpu::new(false);

        let inst = Inst::U(UInst::LUI, UFormat { rd: 10, imm: 1 });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 4096);

        let inst = Inst::U(UInst::LUI, UFormat { rd: 10, imm: 3 });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 12288);

        let inst = Inst::U(UInst::LUI, UFormat { rd: 10, imm: 0x100 });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 1048576);
    }

    #[test]
    fn lui_max() {
        let mut cpu = Cpu::new(false);
        let inst = Inst::U(UInst::LUI, UFormat {
            rd: 10,
            imm: 0b1111_1111_1111_1111,
        });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 0b1111_1111_1111_1111_0000_0000_0000);
    }

    #[test]
    fn long_jump() {
        // manually test big addresses, since emulator has little memory
        // auipc x5, 0x03000
        // jalr x10, x5, -0x400
        let mut cpu = Cpu::new(false);
        // set pc to 0x40000004
        cpu.pc.set(0x40000004);
        let auipc_inst = Inst::U(UInst::AUIPC, UFormat {
            rd: 5,
            imm: 0x3000,
        });
        // rd = pc - 4 + imm << 12
        // 0x40000004 - 4 + 0x3000000
        // 0x40000000 + 0x3000000
        // 0x43000000
        auipc_inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(5), 0x43000000);

        // manually increment PC since no fetching here
        // pc = 0x40000004 + 4
        cpu.pc.set(cpu.pc.get() + 4);

        // jalr rd, rs1, imm  # rd = pc+4; pc = rs1+imm
        // rd = 0x40000008; pc = 0x43000000 + (-0x400i32)
        let jarl_inst = Inst::I(
            IInst::Jalr,
            IFormat {
                rd: 10,
                funct3: 0,
                rs1: 5,
                imm: -0x400i32 as u32
            }
        );
        jarl_inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 0x40000008);
        assert_eq!(cpu.pc.get(), 0x42fffc00);
    }
}
