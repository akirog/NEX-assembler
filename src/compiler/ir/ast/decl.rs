use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::ast::block::AstBlock;
use crate::compiler::ir::types::unresolved_type::*;


pub struct AstDecl<'a> {
    pub kind: AstDeclKind<'a>,
    pub span: Span<'a>,
}

impl<'a> Debug for AstDecl<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for AstDecl<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstDeclKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type, body)
            },
            AstDeclKind::TypeDeclaration { name, new_type } => {
                write!(f, "type {} = {};", name, new_type)
            },
            AstDeclKind::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for field in fields {
                    write!(f, " {}", field)?;
                }
                write!(f, "}}")
            },
        }
    }
}

impl<'a> AstDecl<'a> {
    pub fn new(span: Span<'a>, kind: AstDeclKind<'a>) -> AstDecl<'a> {
        AstDecl { span, kind }
    }
}




pub enum AstDeclKind<'a> {
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