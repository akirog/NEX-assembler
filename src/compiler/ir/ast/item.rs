use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::ast::block::AstBlock;
use crate::compiler::ir::types::written_type::*;



#[derive(Clone)]
pub struct AstItem {
    pub kind: AstItemKind,
    pub span: Span,
}

impl Debug for AstItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for AstItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type.kind, body)
            },
            AstItemKind::TypeDeclaration { name, new_type } => {
                write!(f, "type {} = {};", name, new_type.kind)
            },
            AstItemKind::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for field in fields {
                    write!(f, " {}", field)?;
                }
                write!(f, "}}")
            },
            AstItemKind::EnumDefinition { name, variants } => {
                write!(f, "enum {} {{", name)?;
                for variant in variants {
                    write!(f, "\t{}", variant)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl AstItem {
    pub fn new(span: Span, kind: AstItemKind) -> AstItem {
        AstItem { span, kind }
    }
}




#[derive(Clone)]
pub enum AstItemKind {
    /// A function declaration
    FunctionDeclaration {
        name: String,
        ret_type: WrittenType,
        arguments: Vec<WrittenField>,
        body: AstBlock
    },

    /// Type declaration
    TypeDeclaration {
        name: String,
        new_type: WrittenType,
    },

    /// Struct definition
    StructDefinition {
        name: String,
        fields: Vec<WrittenField>,
    },

    /// Enum definition
    EnumDefinition {
        name: String,
        variants: Vec<String>,
    }
}