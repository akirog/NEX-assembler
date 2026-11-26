use crate::compiler::errors::Span;
use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::hir::SymbolId;

#[derive(Clone)]
pub struct WrittenField {
    pub name: String,
    pub ty: WrittenType,
    pub span: Span,
}

impl Debug for WrittenField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for WrittenField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

#[derive(Clone)]
pub struct HirWrittenField {
    pub id: SymbolId,
    pub ty: WrittenType,
    pub span: Span,
}

impl Debug for HirWrittenField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Display for HirWrittenField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.id, self.ty)
    }
}

#[derive(Clone, Debug)]
pub struct WrittenType {
    pub span: Span,
    pub kind: WrittenTypeKind,
}

impl Display for WrittenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Clone)]
pub enum WrittenTypeKind {
    Name(String),
    Pointer(Box<WrittenType>),
    Slice(Box<WrittenType>),
    Array { ty: Box<WrittenType>, length: Option<usize> },
    FuncPtr { ret_type: Box<WrittenType>, params: Vec<WrittenType> },
}


impl Debug for WrittenTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WrittenTypeKind::Name(name) => { write!(f, "{}", name) }
            WrittenTypeKind::Pointer(inner) => { write!(f, "*{}", inner.kind) }
            WrittenTypeKind::Slice(inner) => { write!(f, "[{}]", inner.kind) }
            WrittenTypeKind::Array { ty: inner_type, length } => { write!(f, "[{}; {:?}]", inner_type.kind, length) }
            WrittenTypeKind::FuncPtr { ret_type, params, .. } => { write!(f, "fn({:?}) -> {}", params, ret_type.kind) }
        }
    }
}

impl Display for WrittenTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
