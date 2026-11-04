use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::types::written_type::*;


#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct TypeId(uid::Id<u16>);
impl TypeId {
    pub fn new() -> TypeId {
        TypeId(uid::Id::new())
    }
}


#[derive(Eq, Hash, PartialEq, Clone)]
pub enum TypeKind {
    U8, U32, I8, I32,
    Bool, Char,
    Pointer(TypeId),
    Array { ty: TypeId, len: usize },
    FuncPtr { ret_ty: TypeId, params: Vec<TypeId> },
    Function { ret_ty: TypeId, params: Vec<TypeId> },
    /// Gets inferred in the hir -> thir phase
    Unresolved,
    UnresolvedNum,
    /// Void
    None,
}

#[derive(Clone)]
pub struct Type {
    pub(crate) id: TypeId,
}

