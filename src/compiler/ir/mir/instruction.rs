use crate::compiler::ir::mir::TempReg;
use crate::compiler::ir::mir::value::MirValue;
use crate::compiler::ir::operations::Operation;


#[derive(Debug)]
pub struct MirInstruction {
    pub kind: MirInstructionKind
}

#[derive(Debug)]
pub enum MirInstructionKind {
    Branch {
        dest: MirValue,
        val1: MirValue,
        val2: MirValue,
        cond: MirCondition,
    },
    Mov {
        target: TempReg,
        value: u32
    },
    Operation {
        target: TempReg,
        val1: MirValue,
        val2: MirValue,
        op: Operation,
    },
    Load {
        target: TempReg,
        addr: MirValue,
    },
    Store {
        addr: MirValue,
        value: MirValue,
    },
    Label { label: String },
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