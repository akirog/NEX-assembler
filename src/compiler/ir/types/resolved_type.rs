use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::types::unresolved_type::*;


#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct TypeId(uid::Id<u16>);

#[derive(Clone, PartialEq)]
pub struct ResolvedField {
    pub name: String,
    pub field_type: TypeId,
}

impl Debug for ResolvedField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for ResolvedField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.field_type)
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedPrimitive {
    U8,
    U32,
    I8,
    I32,
    Char,
    Bool,
    Void
}

impl ResolvedPrimitive {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "u8" => Some(ResolvedPrimitive::U8),
            "u32" => Some(ResolvedPrimitive::U32),
            "i8" => Some(ResolvedPrimitive::I8),
            "i32" => Some(ResolvedPrimitive::I32),
            "char" => Some(ResolvedPrimitive::Char),
            "bool" => Some(ResolvedPrimitive::Bool),
            "void" => Some(ResolvedPrimitive::Void),
            _ => None
        }
    }

    fn get_size(&self) -> usize {
        match self {
            ResolvedPrimitive::U8 => 1,
            ResolvedPrimitive::U32 => 4,
            ResolvedPrimitive::I8 => 1,
            ResolvedPrimitive::I32 => 4,
            ResolvedPrimitive::Char => 1,
            ResolvedPrimitive::Bool => 1,
            ResolvedPrimitive::Void => 0
        }
    }
}


#[derive(Clone, PartialEq)]
pub enum ResolvedType {
    Primitive(ResolvedPrimitive),
    Alias(TypeId),
    Struct(Vec<ResolvedField>),
    Pointer(TypeId),
    Array { length: usize, inner_type: TypeId },
    Function {ret_type: Box<ResolvedType>, params: Vec<ResolvedType>},
}


impl Debug for ResolvedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::Primitive(name) => { write!(f, "{:?}", name) },
            ResolvedType::Struct(fields) => {
                write!(f, "Struct {{\n")?;
                for field in fields {
                    write!(f, "\t{},\n", field)?;
                }
                write!(f, "}}")
            },
            ResolvedType::Pointer(inner) => { write!(f, "*{}", inner) },
            ResolvedType::Unresolved(inner) => { write!(f, "U{}", inner) },
            ResolvedType::Array { inner_type, length } => { write!(f, "[{}; {:?}]", inner_type, length) },
            ResolvedType::Function { ret_type, params, .. } => { write!(f, "fn({:?}) -> {}", params, ret_type) },
        }
    }
}

impl Display for ResolvedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl ResolvedType {
    pub fn get_size(&mut self) -> usize {
        match self {
            ResolvedType::Primitive(prim) => prim.get_size(),
            ResolvedType::Struct(fields) => {
                let mut size = 0;
                for field in fields {
                    size += field.field_type.get_size();
                }
                size
            },
            ResolvedType::Pointer(_inner) => { ResolvedPrimitive::U32.get_size() },
            ResolvedType::Unresolved(_inner) => { panic!("Attempted to get size of unresolved type") },
            ResolvedType::Array { inner_type, length } => {
                inner_type.get_size() * *length
            },
            ResolvedType::Function { ret_type: _, params: _ } => {
                panic!("Attempted to get size of function type.")
            },
        }
    }

    pub fn indexable(&self) -> bool {
        match self {
            ResolvedType::Array { inner_type, length } => true,
            _ => false
        }
    }
    
    pub fn dereferenceable(&self) -> bool {
        match self {
            ResolvedType::Array { inner_type, length } => true,
            ResolvedType::Pointer(_) => true,
            _ => false
        }
    }
    
    pub fn dereference(&self) -> Option<&ResolvedType> {
        match self {
            ResolvedType::Array { inner_type, length } => Some(inner_type),
            ResolvedType::Pointer( inner_type ) => Some(inner_type),
            _ => None
        }
    }
}


#[derive(Debug)]
pub enum TypeKind<'a> {
    Alias(UnresolvedType),
    Struct {
        fields: Vec<UnresolvedField<'a>>,
    },
    Enum {
        variants: Vec<EnumVariant>,
    }
}

impl<'a> Display for TypeKind<'a> {
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
            TypeKind::Enum { variants: _ } => { write!(f, "Display not implemented for enums") }
        }
    }
}


#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub values: Vec<UnresolvedType>,
}

