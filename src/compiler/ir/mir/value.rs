use std::fmt::Display;
use crate::compiler::ir::mir::MirInstruction;
use crate::compiler::ir::operations::Operation;

#[derive(Debug, Clone)]
pub enum Address {
    Absolute(TempReg),
    FrameOffset(usize),
}

impl Address {
    pub fn add(&self, change: i32, instructions: &mut Vec<MirInstruction>) -> Address {
        match self {
            Address::FrameOffset(addr) => {
                Address::FrameOffset((*addr as i32 - change) as usize)
            },
            Address::Absolute(reg) => {
                let new_reg = TempReg::new();
                instructions.push(MirInstruction::Operation {
                    target: new_reg,
                    val1: MirValue::Register(reg.clone()),
                    val2: MirValue::Const(Constant::Number(1)),
                    op: Operation::Add
                });
                
                Address::Absolute(new_reg)
            }
        }
    }
}



#[derive(Debug, Clone)]
pub enum MirValue {
    Const(Constant),
    Register(TempReg),
    Load(Address),
    FrameOffsetAddress(usize),
}

impl Display for MirValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirValue::Const(c) => write!(f, "{:?}", c),
            MirValue::Register(r) => write!(f, "{:?}", r),
            MirValue::Load(l) => write!(f, "{:?}", l),
            MirValue::FrameOffsetAddress(addr) => write!(f, "{}", addr),
        }
    }
}


#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct TempReg(uid::Id<u32>);

impl TempReg {
    pub fn new() -> TempReg {
        TempReg(uid::Id::new())
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Number(u32),
    Label(String),
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Number(n) => write!(f, "{}", n),
            Constant::Label(l) => write!(f, "{}", l),
        }
    }
}

