use std::collections::HashMap;
use crate::compiler::errors::CompilerError;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::{TypeId, WrittenField};


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructDefinition {
    pub name: String,
    pub field_names: Vec<String>,
    pub field_types: Vec<TypeId>,
    pub field_idx_from_name: HashMap<String, usize>,
}


impl<'a> StructDefinition {
    pub fn empty(name: String) -> Self {
        Self {
            name,
            field_names: Vec::new(),
            field_types: Vec::new(),
            field_idx_from_name: HashMap::new(),
        }
    }

    pub fn field_type_from_name(&self, name: &str) -> Option<TypeId> {
        let idx = self.field_idx_from_name.get(name)?;
        Some(self.field_types[*idx])
    }

    pub fn insert(&mut self, field_name: &str, field_ty: TypeId) {
        if self.field_names.contains(&field_name.to_string()) {
            panic!("Attempted to insert duplicate field name");
        }

        let idx = self.field_names.len();
        self.field_types.push(field_ty);
        self.field_names.push(field_name.to_string());
        self.field_idx_from_name.insert(field_name.to_string(), idx);
    }

    pub fn from_written_types(name: String, fields: &Vec<WrittenField<'a>>, type_table: &mut TypeTable) -> Result<StructDefinition, CompilerError<'a>> {
        let mut defintion = StructDefinition::empty(name);

        for field in fields {
            let field_id = type_table.get_written_type_id(&field.ty)?;

            defintion.insert(&field.name, field_id)
        }

        Ok(defintion)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct StructId(pub usize);



pub struct StructLayout {
    pub offsets: Vec<usize>,
    pub types: Vec<TypeId>,
    pub sizes: Vec<usize>,
}

impl StructLayout {
    pub fn new(offsets: Vec<usize>, types: Vec<TypeId>, sizes: Vec<usize>) -> StructLayout {
        StructLayout {
            offsets,
            types,
            sizes,
        }
    }

    pub fn field_count(&self) -> usize {
        self.offsets.len()
    }
}