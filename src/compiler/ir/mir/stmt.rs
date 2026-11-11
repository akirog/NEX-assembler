use std::fmt::{write, Display};
use crate::compiler::ir::mir::MirBlock;


#[derive(Debug)]
pub enum MirStmt {
    Block(Box<MirBlock>),
}

impl Display for MirStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirStmt::Block(block) => write!(f, "{}", block),
        }
    }
}