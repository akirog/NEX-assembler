use crate::compiler::assembly_generation::register_manager::{Register, RegisterManager};
use crate::compiler::ir::mir::MirInstruction;
use crate::compiler::ir::operations::ImmOperation;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Address {
    Absolute(Register),
    BasePointerOffset(i32),
    StackPointerOffset(usize),
}

impl Address {
    pub fn add(&mut self, change: i32, instructions: &mut Vec<MirInstruction>) {
        match self {
            Address::BasePointerOffset(addr) => {
                *self = Address::BasePointerOffset(*addr - change)
            },
            Address::Absolute(reg) => {
                let op = if change > 0 {
                    ImmOperation::Add
                } else {
                    ImmOperation::Subtract
                };

                instructions.push(MirInstruction::ImmOperation {
                    target: reg.clone(),
                    left: reg.clone(),
                    right: Constant::Number(change as u32),
                    op
                });
                
                *self = Address::Absolute(*reg)
            },
            Address::StackPointerOffset(addr) => {
                *self = Address::StackPointerOffset((*addr as i32 + change) as usize)
            }
        }
    }

    pub fn free_if_reg(self, register_manager: &mut RegisterManager) {
        match self {
            Address::Absolute(reg) => { register_manager.free_temp_reg(reg) },
            _ => {}
        }
    }
}



#[derive(Debug)]
pub enum MirValue {
    Const(Constant),
    Register(Register),
    Load(Address),
    FrameOffsetAddress(i32),
    StackPointerOffset(usize),
}

impl Display for MirValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirValue::Const(c) => write!(f, "{:?}", c),
            MirValue::Register(r) => write!(f, "{}", r),
            MirValue::Load(l) => write!(f, "{:?}", l),
            MirValue::FrameOffsetAddress(addr) => write!(f, "bp - {}", addr),
            MirValue::StackPointerOffset(addr) => write!(f, "sp + {}", addr),
        }
    }
}


impl MirValue {
    /// Moves self so value cant be accidentally used again
    pub fn free_if_reg(self, reg_manager: &mut RegisterManager) {
        match self {
            MirValue::Register(reg) => reg_manager.free_temp_reg(reg),
            _ => return
        }
    }

    pub fn ensure_in_reg(&mut self, instructions: &mut Vec<MirInstruction>, register_manager: &mut RegisterManager) -> Register {
        let reg = match self {
            MirValue::Const(constant) => {
                let temp_reg = register_manager.get_temp_reg();

                instructions.push(MirInstruction::Mov {
                    target: temp_reg,
                    value: constant.clone()
                });

                temp_reg
            },
            MirValue::Load(addr) => {
                let temp_reg = register_manager.get_temp_reg();
                instructions.push( MirInstruction::Load {
                    target: temp_reg,
                    addr: addr.clone()
                });
                temp_reg
            },
            MirValue::Register(reg) => {
                *reg
            },
            MirValue::FrameOffsetAddress(offset) => {
                let reg = register_manager.get_temp_reg();
                instructions.push( MirInstruction::AsmText(
                    format!("subi {reg}, bp, {offset}")
                ));
                reg
            }
            MirValue::StackPointerOffset(offset) => {
                let reg = register_manager.get_temp_reg();
                instructions.push( MirInstruction::AsmText(
                    format!("addi {reg}, sp, {offset}")
                ));
                reg
            }
        };

        *self = MirValue::Register(reg);

        reg
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

