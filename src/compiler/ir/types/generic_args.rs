use std::collections::HashMap;
use crate::compiler::errors::CompilerError;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::types::{TypeId, WrittenField, WrittenType};
use crate::compiler::ir::types::type_table::TypeTable;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GenericArgs {
    pub names: Vec<String>,
    pub types: Vec<TypeId>,
}

impl<'a> GenericArgs {
    pub fn new() -> GenericArgs {
        GenericArgs {
            names: Vec::new(),
            types: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: &String, ty: TypeId) {
        if self.names.contains(name) {
            panic!()
        }

        self.names.push(name.clone());
        self.types.push(ty);
    }

    pub fn names(&self) -> &[String] {
        &self.names[..]
    }

    pub fn types(&self) -> &[TypeId] {
        &self.types[..]
    }

    pub fn names_types(&self) -> Vec<(&String, &TypeId)> {
        self.names[..].iter().zip(self.types.iter()).collect::<Vec<_>>()
    }

    pub fn from_written_types(fields: &Vec<WrittenField<'a>>, type_table: &mut TypeTable) -> Result<GenericArgs, CompilerError<'a>> {
        let mut args = GenericArgs::new();

        for field in fields {
            let field_id = type_table.get_written_type_id(&field.ty)?;

            args.insert(&field.name, field_id)
        }

        Ok(args)
    }
}