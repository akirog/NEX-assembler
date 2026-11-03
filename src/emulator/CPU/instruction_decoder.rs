use crate::emulator::CPU::emu::Emulator;

pub struct InstructionDecoder {}

impl InstructionDecoder {
    pub fn new() -> InstructionDecoder {
        InstructionDecoder {}
    }

    pub fn get_instruction(&self, emu: &Emulator) -> CpuInstruction {
        let instruction = emu.memory_manager.load_32(emu.instruction_pointer);
        let opcode: u8 = (instruction >> 26 & 0xFF) as u8;

        let kind = match opcode {
            0..4 | 25 => {
                let flags = (instruction & 0x7FF) as u16;

                let reg_op = match opcode {
                    0 => RegOp::Arith(
                        match flags {
                            0 => RegArithKind::ADD,
                            1 => RegArithKind::SUB,
                            2 => RegArithKind::AND,
                            3 => RegArithKind::OR,
                            4 => RegArithKind::XOR,
                            5 => RegArithKind::NEG,
                            6 => RegArithKind::SHL,
                            7 => RegArithKind::SHR,
                            8 => RegArithKind::MUL,
                            9 => RegArithKind::DIV,
                            10 => RegArithKind::MOD,
                            11 => RegArithKind::LT,
                            12 => RegArithKind::LTE,
                            13 => RegArithKind::EQ,
                            14 => RegArithKind::NE,
                            _ => panic!("Invalid arith op"),
                        }
                    ),
                    1 | 2 => RegOp::Jump(
                        match opcode {
                            1 => RegJumpKind::Jump,
                            2 => RegJumpKind::JumpLink,
                            _ => unreachable!(),
                        }
                    ),
                    3 => RegOp::Io(
                        match flags {
                            0 => RegIoKind::Halt,
                            1 => RegIoKind::Keyboard,
                            2 => RegIoKind::Screen,
                            3 => RegIoKind::Rom,
                            4 => RegIoKind::SSDRead,
                            5 => RegIoKind::SSDWrite,
                            _ => panic!("Invalid io op"),
                        }
                    ),
                    25 => RegOp::Int,
                    _ => unreachable!(),
                };
                CpuInstructionKind::Reg {
                    op: reg_op,
                    src1: (instruction >> 21 & 0x1F) as u8,
                    src2: (instruction >> 16 & 0x1F) as u8,
                    dst: (instruction >> 11 & 0x1F) as u8,
                    flags,
                }
            },
            4..23 => {
                let imm_op = match opcode {
                    4..8 => ImmOp::Mem(
                        ImmMemFlags {
                            store: (opcode & 1) > 0,
                            byte:  (opcode & 2) > 0,
                        },
                    ),
                    8..16 => ImmOp::Arith(
                        match opcode {
                            8 => ImmArithKind::ADD,
                            9 => ImmArithKind::ADDHI,
                            10 => ImmArithKind::OR,
                            11 => ImmArithKind::SUB,
                            12 => ImmArithKind::SHL,
                            13 => ImmArithKind::SHR,
                            14 => ImmArithKind::MUL,
                            15 => ImmArithKind::DIV,
                            _ => unreachable!(),
                        }
                    ),
                    16..23 => ImmOp::Branch(
                        match opcode {
                            16 => ImmBranchKind::Unconditional,
                            17 => ImmBranchKind::Greater,
                            18 => ImmBranchKind::Less,
                            19 => ImmBranchKind::Equal,
                            20 => ImmBranchKind::NotEqual,
                            21 => ImmBranchKind::Above,
                            22 => ImmBranchKind::Below,
                            _ => unreachable!(),
                        }
                    ),
                    _ => unreachable!(),
                };
                CpuInstructionKind::Imm {
                    op: imm_op,
                    dst: (instruction >> 21 & 0x1F) as u8,
                    src1: (instruction >> 16 & 0x1F) as u8,
                    imm: (instruction & 0xFFFF) as u16,
                }
            },
            23 | 24 => {
                CpuInstructionKind::Jmp {
                    kind: match opcode {
                        23 => JmpKind::Jump,
                        24 => JmpKind::JumpLink,
                        _ => unreachable!(),
                    },
                    addr: instruction & 0x3FFFFFF,
                }
            },
            // For unexpected instructions we just give a no op
            _ => CpuInstructionKind::Reg {
                op: RegOp::Arith(RegArithKind::ADD),
                src1: 0,
                src2: 0,
                dst: 0,
                flags: 0,
            },
        };

        CpuInstruction {
            kind,
        }
    }
}



#[derive(Debug)]
pub struct CpuInstruction {
    pub kind: CpuInstructionKind,
}


#[derive(Debug)]
pub enum CpuInstructionKind {
    Reg { op: RegOp, src1: u8, src2: u8, dst: u8, flags: u16 },
    Imm { op: ImmOp, dst: u8, src1: u8, imm: u16 },
    Jmp { kind: JmpKind, addr: u32 },
}

#[derive(Debug)]
pub enum RegOp {
    Arith(RegArithKind),
    Jump(RegJumpKind),
    Io(RegIoKind),
    Int,
}

#[derive(Debug)]
pub enum RegArithKind {
    ADD,
    SUB,
    AND,
    OR,
    XOR,
    NEG,
    SHL,
    SHR,
    MUL,
    DIV,
    MOD,
    LT,
    LTE,
    EQ,
    NE
}

#[derive(Debug)]
pub enum RegJumpKind {
    Jump,
    JumpLink,
}

#[derive(Debug)]
pub enum RegIoKind {
    Halt,
    Keyboard,
    Screen,
    Time,
    Rom,
    SSDRead,
    SSDWrite,
}


#[derive(Debug)]
pub enum ImmOp {
    Arith(ImmArithKind),
    Branch(ImmBranchKind),
    Mem(ImmMemFlags),
}

#[derive(Debug)]
pub struct ImmMemFlags {
    pub store: bool,
    pub byte: bool,
}

#[derive(Debug)]
pub enum ImmArithKind {
    ADD,
    ADDHI,
    OR,
    SUB,
    SHL,
    SHR,
    MUL,
    DIV
}

#[derive(Debug)]
pub enum ImmBranchKind {
    Unconditional,
    Greater,
    Less,
    Equal,
    NotEqual,
    Above,
    Below,
}


#[derive(Debug)]
pub enum JmpKind {
    Jump,
    JumpLink,
}