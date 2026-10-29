use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::types::unresolved_type::*;
use crate::compiler::errors::Span;
use crate::compiler::ir::ast::block::AstBlock;
use crate::compiler::ir::ast::expr::AstExpr;
use crate::compiler::ir::ast::item::AstItem;


#[derive(Clone)]
pub struct AstStmt<'a> {
    pub kind: AstStmtKind<'a>,
    pub span: Span<'a>,
}

impl<'a> AstStmt<'a> {
    pub fn new(span: Span<'a>, kind: AstStmtKind<'a>) -> AstStmt<'a> {
        AstStmt { kind, span }
    }
}

impl<'a> Debug for AstStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for AstStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstStmtKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            AstStmtKind::VariableDeclaration { var_type, name, value } => {
                write!(f, "let {}: {} = {:?}", name, var_type, value)
            },
            AstStmtKind::Item(item) => {
                write!(f, "item {}", item)
            },
            AstStmtKind::Expression(expr) => {
                write!(f, "expr {}", expr)
            },
            AstStmtKind::Semi(expression ) => {
                write!(f, "{};", expression)
            },
        }
    }
}


#[derive(Debug, Clone)]
pub enum AstStmtKind<'a> {
    /// An assignment of an expression to some expression
    Assignment {
        target: Box<AstExpr<'a>>,
        value: Box<AstExpr<'a>>
    },

    /// A variable declaration
    VariableDeclaration {
        name: String,
        value: Option<Box<AstExpr<'a>>>,
        var_type: UnresolvedType,
    },

    /// An item like a function declaration or struct definition
    Item(Box<AstItem<'a>>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<AstExpr<'a>>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<AstExpr<'a>>),
}