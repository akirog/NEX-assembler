use crate::compiler::ir::mir::MirInstruction;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct MirBlock {
    pub label: String,
    pub instructions: Vec<MirInstruction>,
    pub size: u32,

    /// Where in mem to put received reg args
    pub reg_args: Vec<u32>
}

#[derive(Debug)]
pub enum ParamSlot {
    ArgReg,

    /// Sp offset to put this param in
    SpOffset(u32),
}

impl Display for MirBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} size: {}:\n{}", self.label, self.size, self.instructions.iter().map(|x| { x.to_string() }).collect::<Vec<_>>().join("\n"))
    }
}
