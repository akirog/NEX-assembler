use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::ast::block::AstBlock;
use crate::compiler::ir::types::unresolved_type::*;



#[derive(Clone)]
pub struct AstItem<'a> {
    pub kind: AstItemKind<'a>,
    pub span: Span<'a>,
}

impl<'a> Debug for AstItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for AstItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type, body)
            },
            AstItemKind::TypeDeclaration { name, new_type } => {
                write!(f, "type {} = {};", name, new_type)
            },
            AstItemKind::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for field in fields {
                    write!(f, " {}", field)?;
                }
                write!(f, "}}")
            },
        }
    }
}

impl<'a> AstItem<'a> {
    pub fn new(span: Span<'a>, kind: AstItemKind<'a>) -> AstItem<'a> {
        AstItem { span, kind }
    }
}




#[derive(Clone)]
pub enum AstItemKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        name: String,
        ret_type: UnresolvedType,
        arguments: Vec<UnresolvedField<'a>>,
        body: AstBlock<'a>
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