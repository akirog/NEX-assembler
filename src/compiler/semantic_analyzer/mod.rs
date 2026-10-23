pub mod type_table_builder;
pub mod scope_builder;
pub mod symbol_resolver;
pub mod type_checker;

use compiler::ast::BlockNode;

pub struct SemanticAnalyzer<'a> {
    program: BlockNode<'a>,

}


impl<'a> SemanticAnalyzer<'a> {
    pub fn new(program: BlockNode<'a>) -> Self {
        Self {
            program
        }
    }

    pub fn analyze(&mut self) {
        /* Build type table
        Go through all global scope statements and find struct/type declarations and add them to the type table.
        Type table holds key value pairs of type name to type definition
        The type definition holds the types size and kind where kind can be:
        Alias:  ValueType
        Struct: Fields with their name, type and offset
        */


        /* Build scopes declaring symbols with their types at declaration
        Go through each statement storing inner scopes to be resolved for later
        Find all declarations and register their symbols
        Recursively repeat this for the collected statements.
        Giving our scope size as the base offset for those variables.
        */


        /* Assign symbols to identifiers
        Find all identifier use, find their corresponding symbols and assign them.
        */


        /* Assign, infer and enforce types using the data in the assigned symbols
        Find all expressions and recursively find their types.
        Make sure types in binary operations and assignments match.
        */
    }
}