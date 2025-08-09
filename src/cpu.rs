use crate::error::*;
use crate::get_bits;
use crate::inst::*;
use crate::inst_format::*;
use crate::memory::*;
use crate::pc::*;
use crate::regs::*;

enum ProgState {
    Continue,
    Exit(u8),
}

pub struct Cpu {
    pub pc: ProgramCounter,
    pub regs: Registers,
    pub mem: Memory,
    print_debug: bool,
}

impl Cpu {
    pub fn new(print_debug: bool) -> Self {
        Cpu {
            pc: ProgramCounter::new(),
            regs: Registers::new(),
            mem: Memory::new(),
            print_debug,
        }
    }

    pub fn run(&mut self, program: Vec<u8>) -> Result<u8, Error> {
        self.mem.load_program(program);
        for cycle in 0.. {
            match self.emulate_cycle() {
                Ok(ProgState::Exit(code)) => {
                    self.dump_state(cycle);
                    return Ok(code);
                }
                Err(e) => {
                    self.dump_state(cycle);
                    return Err(e);
                }
                // TODO: why is it returning unit type?
                _ => (),
            }
            if self.print_debug {
                self.dump_state(cycle);
            }
        }
        unreachable!("emulator should either run out of instructions or exit using syscall")
    }

    fn dump_state(&self, cycle: usize) {
        eprintln!("CPU dump at cycle {cycle}");
        eprintln!("PC: {}", self.pc.get());
        for i in 0..32 {
            eprintln!("R{i}: {}", self.regs.read(i) as i32)
        }
    }

    // fetches next instruction from memory
    fn fetch(&mut self) -> Result<u32, Error> {
        let pc = self.pc.inc()?;
        Ok(self.mem.read(pc, Size::Word, true))
    }

    // Parses raw byte instruction into correct format.
    // For decode information see docs folder.
    fn decode(&self, raw_inst: u32) -> Result<Inst, Error> {
        // get the lowest 7 bit for the opcode
        let opcode = get_bits!(raw_inst, 0, 6);
        let inst = match opcode {
            0b0110011 => {
                let r_format = RFormat::new(raw_inst);
                let inst = match (r_format.funct3, r_format.funct7) {
                    (0x0, 0x00) => RInst::ADD,
                    (0x0, 0x20) => RInst::SUB,
                    (0x4, 0x00) => RInst::XOR,
                    (0x6, 0x00) => RInst::OR,
                    (0x7, 0x00) => RInst::AND,
                    (0x1, 0x00) => RInst::SLL,
                    (0x5, 0x00) => RInst::SRL,
                    (0x5, 0x20) => RInst::SRA,
                    (0x2, 0x00) => RInst::SLT,
                    (0x3, 0x00) => RInst::SLTU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::R(r_format))),
                };
                Inst::R(inst, r_format)
            }
            0b0010011 => {
                let i_format = IFormat::new(raw_inst);
                let upper_imm = get_bits!(i_format.imm, 5, 11);
                let inst = match (i_format.funct3, upper_imm) {
                    (0x0, _) => ArithIInst::ADDI,
                    (0x4, _) => ArithIInst::XORI,
                    (0x6, _) => ArithIInst::ORI,
                    (0x7, _) => ArithIInst::ANDI,
                    (0x1, 0x00) => ArithIInst::SLLI,
                    (0x5, 0x00) => ArithIInst::SRLI,
                    (0x5, 0x20) => ArithIInst::SRAI,
                    (0x2, _) => ArithIInst::SLTI,
                    (0x3, _) => ArithIInst::SLTIU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::I(i_format))),
                };
                Inst::I(IInst::Arith(inst), i_format)
            }
            0b0000011 => {
                let i_format = IFormat::new(raw_inst);
                let inst = match i_format.funct3 {
                    0x0 => LoadIInst::LB,
                    0x1 => LoadIInst::LH,
                    0x2 => LoadIInst::LW,
                    0x4 => LoadIInst::LBU,
                    0x5 => LoadIInst::LHU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::I(i_format))),
                };
                Inst::I(IInst::Mem(inst), i_format)
            }
            0b1100111 => {
                let i_format = IFormat::new(raw_inst);
                if i_format.funct3 == 0x0 {
                    Inst::I(IInst::Jalr, i_format)
                } else {
                    return Err(Error::InvalidInstFormat(FormatError::I(i_format)));
                }
            }
            0b0100011 => {
                let s_format = SFormat::new(raw_inst);
                let inst = match s_format.funct3 {
                    0x0 => SInst::SB,
                    0x1 => SInst::SH,
                    0x2 => SInst::SW,
                    _ => return Err(Error::InvalidInstFormat(FormatError::S(s_format))),
                };
                Inst::S(inst, s_format)
            }
            0b1100011 => {
                let b_format = BFormat::new(raw_inst);
                let inst = match b_format.funct3 {
                    0x0 => BInst::BEQ,
                    0x1 => BInst::BNE,
                    0x4 => BInst::BLT,
                    0x5 => BInst::BGE,
                    0x6 => BInst::BLTU,
                    0x7 => BInst::BGEU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::B(b_format))),
                };
                Inst::B(inst, b_format)
            }
            0b1101111 => {
                // JAL instruction is the only J-Format instruction
                Inst::J(JFormat::new(raw_inst))
            }
            0b0110111 => Inst::U(UInst::LUI, UFormat::new(raw_inst)),
            0b0010111 => Inst::U(UInst::AUIPC, UFormat::new(raw_inst)),
            0b1110011 => {
                // ecall
                let call = if self.regs.read(17) == 93 {
                    // intercept exit syscall (a7 == 93) to check official risc-v test suite
                    SysCall::Exit(self.regs.read(10) as u8)
                } else {
                    SysCall::Nop
                };
                Inst::SysCall(call)
            }
            0b0001111 => {
                // fence (also necessary for RISC-V tests)
                Inst::SysCall(SysCall::Nop)
            }
            _ => return Err(Error::InvalidOpcode(opcode)),
        };
        Ok(inst)
    }

    fn emulate_cycle(&mut self) -> Result<ProgState, Error> {
        let raw_inst = self.fetch()?;
        if raw_inst == 0 {
            return Err(Error::EndOfInstructions);
        }
        if self.print_debug {
            eprintln!("Instruction: {:032b}", raw_inst);
        }
        let inst = self.decode(raw_inst)?;
        if let Inst::SysCall(SysCall::Exit(code)) = inst {
            return Ok(ProgState::Exit(code))
        }
        inst.execute(self);
        Ok(ProgState::Continue)
    }
}
