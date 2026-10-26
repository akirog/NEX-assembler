use std::fmt::{Debug, Display, Formatter};
use compiler::ast::{BlockNode, UnresolvedField, UnresolvedType};
use compiler::errors::Span;



pub struct DeclNode<'a> {
    pub kind: DeclKind<'a>,
    pub span: Span<'a>,
}

impl<'a> Debug for DeclNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for DeclNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DeclKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type, body)
            },
            DeclKind::TypeDeclaration { name, new_type } => {
                write!(f, "type {} = {};", name, new_type)
            },
            DeclKind::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for field in fields {
                    write!(f, " {}", field)?;
                }
                write!(f, "}}")
            },
        }
    }
}

impl<'a> DeclNode<'a> {
    pub fn new(span: Span<'a>, kind: DeclKind<'a>) -> DeclNode<'a> {
        DeclNode { span, kind }
    }
}




pub enum DeclKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        name: String,
        ret_type: UnresolvedType,
        arguments: Vec<UnresolvedField<'a>>,
        body: BlockNode<'a>
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