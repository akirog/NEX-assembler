use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::thir::block::ThirBlock;
use crate::compiler::ir::types::unresolved_type::*;


#[derive(Clone)]
pub struct ThirItem<'a> {
    pub kind: ThirItemKind<'a>,
    pub span: Span<'a>,
}

impl<'a> Debug for ThirItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for ThirItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ThirItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type, body)
            },
        }
    }
}

impl<'a> ThirItem<'a> {
    pub fn new(span: Span<'a>, kind: ThirItemKind<'a>) -> ThirItem<'a> {
        ThirItem { span, kind }
    }
}




#[derive(Clone)]
pub enum ThirItemKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        name: String,
        ret_type: UnresolvedType,
        arguments: Vec<UnresolvedField<'a>>,
        body: Box<ThirBlock<'a>>
    },
}