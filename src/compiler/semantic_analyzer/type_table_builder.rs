use std::collections::HashMap;
use crate::compiler::*;

pub struct TypeTableBuilder<'a> {
    declarations: &'a Vec<DeclNode<'a>>,

}

impl<'a> TypeTableBuilder<'a> {
    pub fn new(declarations: &'a Vec<DeclNode<'a>>) -> Self {
        Self {
            declarations
        }
    }


    pub fn build(&mut self) -> HashMap<String, TypeDefinition<'a>> {
        let mut type_table: HashMap<String, TypeDefinition> = HashMap::new();

        for node in self.declarations {
            match &node.kind {
                DeclKind::StructDefinition { name, fields } => {
                    let type_definition = TypeDefinition {
                        name: name.clone(),
                        kind: TypeKind::Struct {
                            fields: (*fields).clone()
                        }
                    };

                    type_table.insert(name.clone(), type_definition);
                },
                DeclKind::TypeDeclaration { name, new_type } => {
                    let type_definition = TypeDefinition {
                        name: name.clone(),
                        kind: TypeKind::Alias(new_type.clone())
                    };

                    type_table.insert(name.clone(), type_definition);
                },
                _ => { continue; }
            }
        }

        type_table
    }
}