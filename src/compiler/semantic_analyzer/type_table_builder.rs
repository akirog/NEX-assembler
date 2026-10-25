use std::collections::HashMap;
use compiler::ast::{BlockNode, StmtKind, TypeDefinition, TypeKind};

pub struct TypeTableBuilder<'a> {
    ast: &'a BlockNode<'a>,

}

impl<'a> TypeTableBuilder<'a> {
    pub fn new(ast: &'a BlockNode<'a>) -> Self {
        Self {
            ast
        }
    }


    pub fn build(&mut self) -> HashMap<String, TypeDefinition> {
        let mut type_table: HashMap<String, TypeDefinition> = HashMap::new();

        for node in &self.ast.statements {
            match &node.kind {
                StmtKind::StructDefinition { name, fields } => {
                    let type_definition = TypeDefinition {
                        name: name.clone(),
                        kind: TypeKind::Struct {
                            fields: (*fields).clone()
                        }
                    };

                    type_table.insert(name.clone(), type_definition);
                }
                _ => { continue; }
            }
        }

        type_table
    }
}