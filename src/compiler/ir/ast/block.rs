use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::ast::stmt::*;
use crate::compiler::errors::Span;


#[derive(Debug, Clone)]
pub struct AstBlock {
    pub statements: Vec<AstStmt>,
    pub span: Span,
}

impl AstBlock {
    pub fn new(statements: Vec<AstStmt>, span: Span) -> AstBlock {
        AstBlock {
            statements,
            span,
        }
    }
}

impl<'a> Display for AstBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n")?;
        let mut statements = String::new();
        for stmt in &self.statements {
            statements.push_str(format!("{}\n", stmt).as_str());
        }

        let mut body_str = String::new();
        for stmt in statements.lines() {
            body_str.push_str(format!("   {}\n", stmt).as_str());
        }
        write!(f, "{}", body_str)?;

        write!(f, "}}")
    }
}