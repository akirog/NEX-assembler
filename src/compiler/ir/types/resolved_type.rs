use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::types::written_type::*;


#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct TypeId(uid::Id<u16>);
impl TypeId {
    pub fn new() -> TypeId {
        TypeId(uid::Id::new())
    }
}


pub enum TypeKind {

}

#[derive(Clone)]
pub struct Type {
    pub(crate) id: TypeId,
}

