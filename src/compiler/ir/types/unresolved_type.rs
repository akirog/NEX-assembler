use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::types::resolved_type::TypeKind;

#[derive(Clone)]
pub struct UnresolvedField<'a> {
    pub name: String,
    pub field_type: UnresolvedType,
    pub span: Span<'a>,
}

impl<'a> Debug for UnresolvedField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for UnresolvedField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.field_type)
    }
}

#[derive(Clone, PartialEq)]
pub enum UnresolvedType {
    Primitive(String),
    Pointer(Box<UnresolvedType>),
    Array { length: Option<usize>, inner_type: Box<UnresolvedType> },
    Function {ret_type: Box<UnresolvedType>, params: Vec<UnresolvedType>},
}


impl Debug for UnresolvedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedType::Primitive(name) => { write!(f, "{}", name) }
            UnresolvedType::Pointer(inner) => { write!(f, "*{}", inner) }
            UnresolvedType::Array { inner_type, length } => { write!(f, "[{}; {:?}]", inner_type, length) }
            UnresolvedType::Function { ret_type, params, .. } => { write!(f, "fn({:?}) -> {}", params, ret_type) }
        }
    }
}

impl Display for UnresolvedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


/// Definition, might move to second lowering
#[derive(Debug)]
pub struct TypeDefinition<'a> {
    pub name: String,
    pub kind: TypeKind<'a>,
}

impl<'a> Display for TypeDefinition<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.kind)
    }
}

