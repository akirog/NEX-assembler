use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use compiler::ast::stmt::StmtNode;
use compiler::errors::Span;

#[derive(Debug)]
pub struct BlockNode<'a> {
    pub statements: Vec<StmtNode<'a>>,
    pub span: Span<'a>,
    pub scope: Cell<Option<usize>>,
}

impl<'a> Display for BlockNode<'a> {
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