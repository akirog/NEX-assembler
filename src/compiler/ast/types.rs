use std::fmt::{write, Debug, Display, Formatter};

pub const NONE_TYPE: &'static str = "void";



pub struct TypeDefinition {
    pub name: String,
    pub kind: TypeKind,
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.kind)
    }
}


/// A type definitions kind
pub enum TypeKind {
    Alias(ValueType),
    Struct {
        fields: Vec<Field>,
    },
    Enum {
        variants: Vec<EnumVariant>,
    }
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Alias(value) => write!(f, "{}", value),
            TypeKind::Struct { fields } => {
                write!(f, "struct {{\n")?;
                for field in fields {
                    write!(f, "\t{},\n", field)?;
                }
                write!(f, "}}")
            },
            TypeKind::Enum { variants } => { write!(f, "Display not implemented for enums") }
        }
    }
}


pub struct EnumVariant {
    pub name: String,
    pub values: Vec<ValueType>,
}


#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub field_type: ValueType,
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.field_type)
    }
}



#[derive(Clone)]
pub enum ValueType {
    Primitive(String),
    Pointer(Box<ValueType>),
    Array { length: Option<usize>, inner_type: Box<ValueType> },
    Function {ret_type: Box<ValueType>, params: Vec<ValueType>},
}


impl Debug for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Primitive(name) => { write!(f, "{}", name) }
            ValueType::Pointer(inner) => { write!(f, "*{}", inner) }
            ValueType::Array { inner_type, length } => { write!(f, "[{}; {:?}]", inner_type, length) }
            ValueType::Function { ret_type, params, .. } => { write!(f, "fn({:?}) -> {}", params, ret_type) }
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}