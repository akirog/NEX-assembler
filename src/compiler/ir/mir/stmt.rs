use crate::compiler::ir::mir::{MirBlock, MirInstruction};



#[derive(Debug)]
pub enum MirStmt {
    Block(Box<MirBlock>),
}