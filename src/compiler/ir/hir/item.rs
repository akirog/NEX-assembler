use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::block::HirBlock;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::types::written_type::*;


#[derive(Clone)]
pub struct HirItem {
    pub kind: HirItemKind,
    pub span: Span,
}

impl Debug for HirItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for HirItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HirItemKind::FunctionDeclaration { id, ret_type, arguments, body } => {
                write!(f, "fn {:?}({:?}) -> {} {}", id, arguments, ret_type.kind, body)
            },
        }
    }
}

impl HirItem {
    pub fn new(span: Span, kind: HirItemKind) -> HirItem {
        HirItem { span, kind }
    }
}




#[derive(Clone)]
pub enum HirItemKind {
    /// A function declaration
    FunctionDeclaration {
        id: SymbolId,
        ret_type: WrittenType,
        arguments: Vec<HirWrittenField>,
        body: Box<HirBlock>
    },
}