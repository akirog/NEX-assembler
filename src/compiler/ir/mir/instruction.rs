use std::fmt::{write, Display};
use crate::compiler::ir::mir::{Address, Constant, TempReg};
use crate::compiler::ir::mir::value::MirValue;
use crate::compiler::ir::operations::Operation;

#[derive(Debug)]
pub enum MirInstruction {
    Branch {
        dest: String,
        val1: MirValue,
        val2: MirValue,
        cond: MirCondition,
    },
    Call {
        dest: MirValue,
        args: Vec<MirValue>,
    },
    Ret {
        value: MirValue,
    },
    Mov {
        target: TempReg,
        value: Constant
    },
    Operation {
        target: TempReg,
        val1: MirValue,
        val2: MirValue,
        op: Operation,
    },
    Load {
        target: TempReg,
        addr: Address,
    },
    Store {
        target: Address,
        value: MirValue,
        byte: bool
    },
    Label { label: String },
    Jump { label: String },

    /// Just a list of instructions the assembler indents for readability
    Indented {
        instructions: Vec<MirInstruction>,
    }
}


impl Display for MirInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirInstruction::Branch { dest, val1, val2, cond } => {
                write!(f, "b{} {}, {}, {}", cond.assembly_text(), val1, val2, dest)
            },
            MirInstruction::Call { dest, args } => {
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "mov a{i}, {}", arg)?;
                }
                write!(f, "call {}", dest)
            },
            MirInstruction::Ret { value } => {
                write!(f, "mov v0, {}", value)?;
                write!(f, "ret")
            },
            MirInstruction::Mov { target, value } => {
                write!(f, "mov {:?}, {}", target, value)
            },
            MirInstruction::Load { target, addr } => {
                write!(f, "load {:?}, [{:?}]", target, addr)
            },
            MirInstruction::Store { target, value, byte } => {
                write!(f, "store{} [{:?}], {}", if *byte {"b"} else {""}, target, value)
            },
            MirInstruction::Label { label } => {
                write!(f, "{}:", label)
            },
            MirInstruction::Jump { label } => {
                write!(f, "jump {}", label)
            },
            MirInstruction::Indented { instructions } => {
                for instruction in instructions {
                    write!(f, "\t{}", instruction)?;
                }
                Ok(())
            },
            MirInstruction::Operation { target, val1, val2, op } => {
                write!(f, "{} {:?}, {}, {}", op, target, val1, val2,)
            },
        }
    }
}


#[derive(Debug)]
pub enum MirCondition {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Ab,
    Be
}

impl MirCondition {
    pub fn assembly_text(&self) -> String {
        match self {
            MirCondition::Eq => "eq".to_string(),
            MirCondition::Ne => "ne".to_string(),
            MirCondition::Gt => "gt".to_string(),
            MirCondition::Ge => "ge".to_string(),
            MirCondition::Lt => "lt".to_string(),
            MirCondition::Le => "le".to_string(),
            MirCondition::Ab => "ab".to_string(),
            MirCondition::Be => "be".to_string()
        }
    }
}