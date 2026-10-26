use std::fmt::{write, Debug, Display, Formatter};
use compiler::errors::{CompilerError, Span};

pub const NONE_TYPE: &'static str = "void";



pub struct TypeDefinition<'a> {
    pub name: String,
    pub kind: TypeKind<'a>,
}

impl<'a> Display for TypeDefinition<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.kind)
    }
}


/// A type definitions kind
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
            TypeKind::Enum { variants } => { write!(f, "Display not implemented for enums") }
        }
    }
}


pub struct EnumVariant {
    pub name: String,
    pub values: Vec<UnresolvedType>,
}


// ============ Unresolved type ===========
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

#[derive(Clone)]
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




// ============ Resolved type ===========
#[derive(Clone)]
pub struct ResolvedField {
    pub name: String,
    pub field_type: ResolvedType,
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


#[derive(Clone, Debug)]
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


#[derive(Clone)]
pub enum ResolvedType {
    Primitive(ResolvedPrimitive),
    Struct(Vec<ResolvedField>),
    Pointer(Box<ResolvedType>),
    Array { length: usize, inner_type: Box<ResolvedType> },
    Function {ret_type: Box<ResolvedType>, params: Vec<ResolvedType>},
}


impl Debug for ResolvedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::Primitive(name) => { write!(f, "{:?}", name) },
            ResolvedType::Struct(fields) => {
                write!(f, "Struct {{")?;
                for field in fields {
                    write!(f, "\t{},\n", field)?;
                }
                write!(f, "}}")
            }
            ResolvedType::Pointer(inner) => { write!(f, "*{}", inner) },
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
    pub fn get_size(&self) -> usize {
        match self {
            ResolvedType::Primitive(prim) => prim.get_size(),
            ResolvedType::Struct(fields) => {
                let mut size = 0;
                for field in fields {
                    size += field.field_type.get_size();
                }
                size
            },
            ResolvedType::Pointer(inner) => { ResolvedPrimitive::U32.get_size() },
            ResolvedType::Array { inner_type, length } => {
                inner_type.get_size() * length
            },
            ResolvedType::Function { ret_type, params } => {
                panic!("Attempted to get size of function type.")
            },
        }
    }
}
