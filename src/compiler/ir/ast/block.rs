use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::ast::stmt::*;
use crate::compiler::errors::Span;


#[derive(Debug, Clone)]
pub struct AstBlock<'a> {
    pub statements: Vec<AstStmt<'a>>,
    pub span: Span<'a>,
}

impl<'a> AstBlock<'a> {
    pub fn new(statements: Vec<AstStmt<'a>>, span: Span<'a>) -> AstBlock<'a> {
        AstBlock {
            statements,
            span,
        }
    }
}

impl<'a> Display for AstBlock<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n")?;
        let mut statements = String::new();
        for stmt in &self.statements {
            statements.push_str(format!("{}\n", stmt.to_string()).as_str());
        }

        let mut body_str = String::new();
        for stmt in statements.lines() {
            body_str.push_str(format!("   {}\n", stmt).as_str());
        }
        write!(f, "{}", body_str)?;

        write!(f, "}}")
    }
}