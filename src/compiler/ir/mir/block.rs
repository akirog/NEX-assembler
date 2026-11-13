use std::fmt::{Display, Formatter};
use crate::compiler::assembly_generation::register_manager::Register;
use crate::compiler::ir::mir::{CallArg, MirInstruction};

#[derive(Debug)]
pub struct MirBlock {
    pub label: String,
    pub instructions: Vec<MirInstruction>,
    pub size: u32,
    pub args: Vec<FuncArg>
}

#[derive(Debug)]
pub enum FuncArg {
    Address {
        /// SP + 8 + this
        arg_addr: u32,
        to: u32
    },
    Register {
        arg_reg: u32,
        to: u32
    }
}

impl Display for MirBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} size: {}:\n{}", self.label, self.size, self.instructions.iter().map(|x| { x.to_string() }).collect::<Vec<_>>().join("\n"))
    }
}