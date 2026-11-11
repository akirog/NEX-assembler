use std::fmt::{Display, Formatter};
use crate::compiler::ir::mir::MirInstruction;

#[derive(Debug)]
pub struct MirBlock {
    pub label: String,
    pub instructions: Vec<MirInstruction>,
    pub size: u32,
    // Where args in a0 - a4 should be placed in memory
    pub args: Vec<u32>
}

impl Display for MirBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} size: {}:\n{}", self.label, self.size, self.instructions.iter().map(|x| { x.to_string() }).collect::<Vec<_>>().join("\n"))
    }
}