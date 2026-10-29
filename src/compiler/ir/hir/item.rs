use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::block::HirBlock;
use crate::compiler::ir::types::unresolved_type::*;


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
            HirItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type, body)
            },
            HirItemKind::TypeDeclaration { name, new_type } => {
                write!(f, "type {} = {};", name, new_type)
            },
            HirItemKind::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for field in fields {
                    write!(f, " {}", field)?;
                }
                write!(f, "}}")
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
        name: String,
        ret_type: UnresolvedType,
        arguments: Vec<UnresolvedField<'a>>,
        body: HirBlock<'a>
    },

    /// Type declaration
    TypeDeclaration {
        name: String,
        new_type: UnresolvedType,
    },

    /// Struct definition
    StructDefinition {
        name: String,
        fields: Vec<UnresolvedField<'a>>,
    },
}