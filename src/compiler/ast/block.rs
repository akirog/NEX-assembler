use std::fmt::{Debug, Display, Formatter};
use compiler::ast::stmt::StmtNode;
use compiler::errors::Span;

#[derive(Debug)]
pub struct BlockNode<'a> {
    pub statements: Vec<StmtNode<'a>>,
    pub span: Span<'a>,
}

impl<'a> Display for BlockNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n")?;
        for stmt in &self.statements {
            write!(f, "{}\n", stmt)?;
        }
        write!(f, "}}")
    }
}