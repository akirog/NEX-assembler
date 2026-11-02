use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::types::resolved_type::TypeKind;

#[derive(Clone)]
pub struct WrittenField<'a> {
    pub name: String,
    pub field_type: WrittenType,
    pub span: Span<'a>,
}

impl<'a> Debug for WrittenField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for WrittenField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.field_type)
    }
}

#[derive(Clone, PartialEq)]
pub enum WrittenType {
    Name(String),
    Pointer(Box<WrittenType>),
    Array { length: Option<usize>, inner_type: Box<WrittenType> },
    Function {ret_type: Box<WrittenType>, params: Vec<WrittenType>},
}


impl Debug for WrittenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WrittenType::Name(name) => { write!(f, "{}", name) }
            WrittenType::Pointer(inner) => { write!(f, "*{}", inner) }
            WrittenType::Array { inner_type, length } => { write!(f, "[{}; {:?}]", inner_type, length) }
            WrittenType::Function { ret_type, params, .. } => { write!(f, "fn({:?}) -> {}", params, ret_type) }
        }
    }
}

impl Display for WrittenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
