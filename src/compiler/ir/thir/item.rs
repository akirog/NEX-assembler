use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::thir::block::ThirBlock;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::{TypeId, TypeKind};
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
            ThirItemKind::FunctionDeclaration { id, ty, body, .. } => {
                let TypeKind::Function { params: param_types, .. } = type_table.get_type(ty) else { panic!("Function decl type is not function type") };
                let args_string = param_types.iter().map(|t| {
                    type_table.type_as_string(t)
                }).collect::<Vec<String>>().join(", ");

                format!("fn {}({:?}) {}", id.0, args_string, body.pretty_print(type_table))
            },
        }
    }

}




#[derive(Clone, Debug)]
pub enum ThirItemKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        id: SymbolId,
        param_ids: Vec<SymbolId>,
        ty: TypeId,
        body: Box<ThirBlock<'a>>
    },
}