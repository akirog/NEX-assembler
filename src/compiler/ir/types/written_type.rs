use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::types::resolved_type::TypeKind;

#[derive(Clone)]
pub struct WrittenField<'a> {
    pub name: String,
    pub ty: WrittenType<'a>,
    pub span: Span<'a>,
}

impl<'a> Debug for WrittenField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for WrittenField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.ty)
    }
}

#[derive(Clone, Debug)]
pub struct WrittenType<'a> {
    pub span: Span<'a>,
    pub kind: WrittenTypeKind<'a>,
}

#[derive(Clone)]
pub enum WrittenTypeKind<'a> {
    Name(String),
    Pointer(Box<WrittenType<'a>>),
    Array { ty: Box<WrittenType<'a>>, length: Option<usize> },
    FuncPtr { ret_type: Box<WrittenType<'a>>, params: Vec<WrittenType<'a>> },
}


impl<'a> Debug for WrittenTypeKind<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WrittenTypeKind::Name(name) => { write!(f, "{}", name) }
            WrittenTypeKind::Pointer(inner) => { write!(f, "*{}", inner.kind) }
            WrittenTypeKind::Array { ty: inner_type, length } => { write!(f, "[{}; {:?}]", inner_type.kind, length) }
            WrittenTypeKind::FuncPtr { ret_type, params, .. } => { write!(f, "fn({:?}) -> {}", params, ret_type.kind) }
        }
    }
}

impl Display for WrittenTypeKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
