use std::fmt::{Display, Formatter};
use crate::compiler::ir::mir::MirInstruction;

#[derive(Debug)]
pub struct MirBlock {
    pub label: String,
    pub instructions: Vec<MirInstruction>,
    pub size: u32
}

impl Display for MirBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} size: {}:\n{:?}", self.label, self.size, self.instructions)
    }
}