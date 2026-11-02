use std::collections::HashMap;
use crate::compiler::ir::types::{TypeId, TypeKind};

pub struct TypeTable {
    types: HashMap<TypeId, TypeKind>,
    reverse_types: HashMap<TypeKind, TypeId>,
}


impl TypeTable {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            reverse_types: HashMap::new(),
        }
    }
}