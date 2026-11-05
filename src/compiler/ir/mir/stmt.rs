use crate::compiler::ir::mir::{MirBlock, MirInstruction};


#[derive(Debug)]
pub enum MirStmt {
    Instruction(MirInstruction),
    Block(Box<MirBlock>),
}