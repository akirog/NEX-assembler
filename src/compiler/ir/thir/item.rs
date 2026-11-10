use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::thir::block::ThirBlock;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::written_type::*;
use std::fmt::Debug;


#[derive(Clone, Debug)]
pub struct ThirItem<'a> {
    pub kind: ThirItemKind<'a>,
    pub span: Span<'a>,
}



impl<'a> ThirItem<'a> {
    pub fn new(span: Span<'a>, kind: ThirItemKind<'a>) -> ThirItem<'a> {
        ThirItem { span, kind }
    }

    pub fn pretty_print(&self, type_table: &TypeTable) -> String {
        match &self.kind {
            ThirItemKind::FunctionDeclaration { id, arguments, body } => {
                format!("fn {}({:?}) {}", id.0, arguments, body.pretty_print(type_table))
            },
        }
    }

}




#[derive(Clone, Debug)]
pub enum ThirItemKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        id: SymbolId,
        arguments: Vec<HirWrittenField<'a>>,
        body: Box<ThirBlock<'a>>
    },
}