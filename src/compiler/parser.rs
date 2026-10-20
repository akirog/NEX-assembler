use compiler::ast::traits::Walkable;
use compiler::ast::nodes::*;
use compiler::errors::CompilerError;
use compiler::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens
        }
    }


    pub fn parse(&mut self) -> Result<(), CompilerError> {
        let mut test_node = ValueNode::new(
            10
        );


        let result =test_node.walk(Box::new(|walkable: &dyn Walkable| -> Result<(), CompilerError> {
            match walkable {
                some => {

                }
            }
        }));

        match result {
            Err(e) => { return Err(e) }
            _ => {}
        }

        Ok(())
    }

}


