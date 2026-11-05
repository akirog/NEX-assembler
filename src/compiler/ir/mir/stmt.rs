use crate::compiler::ir::mir::{MirBlock, MirInstruction, MirInstructionKind};

#[derive(Debug)]
pub struct MirStmt {
    pub kind: MirStmtKind
}

#[derive(Debug)]
pub enum MirStmtKind {
    Instruction(MirInstruction),
    Block(Box<MirBlock>),
}