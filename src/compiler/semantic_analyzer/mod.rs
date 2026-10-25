pub mod type_table_builder;
pub mod scope_builder;
pub mod symbol_resolver;
pub mod type_checker;

use std::collections::HashMap;
use compiler::ast::{BlockNode, TypeDefinition};
use compiler::semantic_analyzer::type_table_builder::TypeTableBuilder;

pub struct SemanticAnalyzer<'a> {
    dump_flags: String,
    program: BlockNode<'a>,
    type_table: HashMap<String, TypeDefinition>,
}


impl<'a> SemanticAnalyzer<'a> {
    pub fn new(program: BlockNode<'a>, dump_flags: String) -> Self {
        Self {
            dump_flags,
            program,
            type_table: HashMap::new(),
        }
    }

    pub fn analyze(&mut self) {
        /*
        Build type table
        Go through all global scope statements and find struct/type declarations and add them to the type table.
        Type table holds key value pairs of type name to type definition.
        The type definition holds the types size and kind where kind can be:
        Alias:  ValueType
        Struct: Fields with their name, type and offset
        */
        let mut type_table_builder = TypeTableBuilder::new(
            &self.program
        );
        self.type_table = type_table_builder.build();
        if self.dump_flags.contains('y') {
            println!("\nType table:");
            for definition in self.type_table.values() {
                println!("{}\n", definition);
            }
        }


        /*
        Build scopes declaring symbols with their types at declaration.
        When assigning types to symbols, the types should also be resolved into pure value types
        Go through each statement storing inner scopes to be resolved for later.
        Find all declarations and register their symbols.
        Recursively repeat this for the collected statements.
        Giving our scope size as the base offset for those variables.
        */


        /*
        Assign symbols to identifiers.
        Find all identifier use, find their corresponding symbols and assign them.
        */


        /*
        Assign, infer and enforce types using the data in the assigned symbols.
        Find all expressions and recursively find their types.
        Make sure types in binary operations and assignments match.
        */
    }
}