use std::fmt::{Display, Formatter};
use crate::compiler::ir::mir::stmt::MirStmt;

#[derive(Debug)]
pub struct MirBlock {
    pub label: String,
    pub statements: Vec<MirStmt>,
    pub size: u32
}

impl Display for MirBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} size: {}:\n{:?}", self.label, self.size, self.statements)
    }
}