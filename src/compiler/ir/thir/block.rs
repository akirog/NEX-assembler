use crate::compiler::errors::Span;
use crate::compiler::ir::thir::stmt::*;
use crate::compiler::ir::types::type_table::TypeTable;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ThirBlock {
    pub statements: Vec<ThirStmt>,
    pub span: Span,
}

impl ThirBlock {
    pub fn new(statements: Vec<ThirStmt>, span: Span) -> ThirBlock {
        ThirBlock {
            statements,
            span,
        }
    }

    pub fn pretty_print(&self, type_table: &TypeTable) -> String {
        let mut result = format!("{{\n");
        let mut statements = String::new();
        for stmt in &self.statements {
            statements.push_str(&format!("{}\n", stmt.pretty_print(type_table).as_str()));
        }

        let mut body_str = String::new();
        for stmt in statements.lines() {
            body_str.push_str(format!("   {}\n", stmt).as_str());
        }
        result.push_str(&format!("{}", body_str));

        result.push_str(&format!("}}"));
        result
    }
}
