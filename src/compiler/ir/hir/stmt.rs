use std::fmt::{Debug, Display, Formatter};
use crate::compiler::ir::types::unresolved_type::*;
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::block::HirBlock;
use crate::compiler::ir::hir::expr::HirExpr;
use crate::compiler::ir::hir::item::HirItem;

#[derive(Clone)]
pub struct HirStmt<'a> {
    pub kind: HirStmtKind<'a>,
    pub span: Span<'a>,
}

impl<'a> HirStmt<'a> {
    pub fn new(span: Span<'a>, kind: HirStmtKind<'a>) -> HirStmt<'a> {
        HirStmt { kind, span }
    }
}

impl<'a> Debug for HirStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for HirStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HirStmtKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            HirStmtKind::VariableDeclaration { var_type, name, value } => {
                write!(f, "let {}: {} = {:?}", name, var_type, value)
            },
            HirStmtKind::Loop { body } => {
                write!(f, "loop {}", body)
            },
            HirStmtKind::Item(item) => {
                write!(f, "item {}", item)
            },
            HirStmtKind::Expression(expr) => {
                write!(f, "expr {}", expr)
            }
            HirStmtKind::Semi(expression ) => {
                write!(f, "{};", expression)
            },
        }
    }
}


#[derive(Debug, Clone)]
pub enum HirStmtKind<'a> {
    /// Unconditional loop
    Loop {
        body: HirBlock<'a>
    },

    /// An assignment of an expression to some expression
    Assignment {
        target: Box<HirExpr<'a>>,
        value: Box<HirExpr<'a>>
    },

    /// A variable declaration
    VariableDeclaration {
        name: String,
        value: Option<Box<HirExpr<'a>>>,
        var_type: UnresolvedType,
    },

    /// An item like a function declaration
    Item(Box<HirItem<'a>>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<HirExpr<'a>>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<HirExpr<'a>>),
}