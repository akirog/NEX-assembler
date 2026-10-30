use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::types::unresolved_type::*;
use crate::compiler::errors::Span;
use crate::compiler::ir::thir::block::ThirBlock;
use crate::compiler::ir::thir::expr::ThirExpr;
use crate::compiler::ir::thir::item::ThirItem;
use crate::compiler::ir::hir::SymbolId;

#[derive(Clone)]
pub struct ThirStmt<'a> {
    pub kind: ThirStmtKind<'a>,
    pub span: Span<'a>,
}

impl<'a> ThirStmt<'a> {
    pub fn new(span: Span<'a>, kind: ThirStmtKind<'a>) -> ThirStmt<'a> {
        ThirStmt { kind, span }
    }
}

impl<'a> Debug for ThirStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for ThirStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ThirStmtKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            ThirStmtKind::VariableDeclaration { var_type, symbol_id, value } => {
                write!(f, "let {}: {} = {:?}", symbol_id, var_type, value)
            },
            ThirStmtKind::Item(item) => {
                write!(f, "item {}", item)
            },
            ThirStmtKind::Expression(expr) => {
                write!(f, "expr {}", expr)
            }
            ThirStmtKind::Semi(expression ) => {
                write!(f, "{};", expression)
            },
        }
    }
}


#[derive(Debug, Clone)]
pub enum ThirStmtKind<'a> {
    /// An assignment of an expression to some expression
    Assignment {
        target: Box<ThirExpr<'a>>,
        value: Box<ThirExpr<'a>>
    },

    /// A variable declaration
    VariableDeclaration {
        symbol_id: SymbolId,
        value: Option<Box<ThirExpr<'a>>>,
        var_type: UnresolvedType,
    },

    /// An item like a function declaration
    Item(Box<ThirItem<'a>>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<ThirExpr<'a>>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<ThirExpr<'a>>),
}