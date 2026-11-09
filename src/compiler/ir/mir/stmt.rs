use crate::compiler::ir::mir::MirBlock;


#[derive(Debug)]
pub enum MirStmt {
    Block(Box<MirBlock>),
}