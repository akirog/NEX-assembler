use crate::compiler::ir::mir::{Address, Constant, Place, TempReg};
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