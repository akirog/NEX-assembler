use std::collections::HashMap;
use compiler::ast::{BlockNode, TypeDefinition};

pub struct TypeTableBuilder<'a> {
    ast: BlockNode<'a>,

}

impl<'a> TypeTableBuilder<'a> {
    pub fn new(ast: BlockNode<'a>) -> Self {
        Self {
            ast
        }
    }


    pub fn build(&mut self) -> HashMap<String, TypeDefinition> {
        let mut type_table: HashMap<String, TypeDefinition> = HashMap::new();

        for node in &self.ast.statements {
            match node {

                _ => { continue; }
            }
        }

        type_table
    }
}