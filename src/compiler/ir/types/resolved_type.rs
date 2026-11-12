use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::compiler::ir::types::generic_args::FieldTypes;

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct TypeId(uid::Id<u16>);
impl TypeId {
    pub fn new() -> TypeId {
        TypeId(uid::Id::new())
    }
}

impl Debug for TypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum TypeKind {
    U8, U32, I8, I32,
    Bool, Char,
    Pointer(TypeId),
    Slice(TypeId),
    Array { ty: TypeId, len: usize },
    FuncPtr { ret_ty: TypeId, params: Vec<TypeId> },
    Function { ret_ty: TypeId, params: Vec<TypeId> },
    Struct { fields: FieldTypes },
    /// Gets inferred in the hir -> thir phase
    Unresolved,
    UnresolvedNum,
    /// Void
    None,
    Error,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Type {
    pub id: TypeId,
}

