use std::fmt::{Display, Formatter};


#[derive(Debug)]
pub enum ValueType {
    Primitive(String),
    Pointer(Box<ValueType>),
    Array { length: Option<usize>, inner_type: Box<ValueType> },
    FunctionCall {ret_type: Box<ValueType>, params: Vec<ValueType>},
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}