use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::block::HirBlock;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::types::written_type::*;


#[derive(Clone)]
pub struct HirItem<'a> {
    pub kind: HirItemKind<'a>,
    pub span: Span<'a>,
}

impl<'a> Debug for HirItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for HirItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HirItemKind::FunctionDeclaration { id, ret_type, arguments, body } => {
                write!(f, "fn {:?}({:?}) -> {} {}", id, arguments, ret_type.kind, body)
            },
        }
    }
}

impl<'a> HirItem<'a> {
    pub fn new(span: Span<'a>, kind: HirItemKind<'a>) -> HirItem<'a> {
        HirItem { span, kind }
    }
}




#[derive(Clone)]
pub enum HirItemKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        id: SymbolId,
        ret_type: WrittenType<'a>,
        arguments: Vec<HirWrittenField<'a>>,
        body: Box<HirBlock<'a>>
    },
}