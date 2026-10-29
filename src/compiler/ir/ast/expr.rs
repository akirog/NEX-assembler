use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::ast::{AstBlock, AstStmt, AstStmtKind};
use crate::compiler::ir::operations::*;
use crate::compiler::ir::types::unresolved_type::*;

#[derive(Clone)]
pub struct AstExpr<'a> {
    pub kind: AstExprKind<'a>,
    pub span: Span<'a>,
}

impl<'a> AstExpr<'a> {
    pub fn new(span: Span<'a>, kind: AstExprKind<'a>) -> AstExpr<'a> {
        AstExpr { kind, span }
    }
}

#[derive(Clone, Debug)]
pub enum AstExprKind<'a> {
    /// Number
    Number(u32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<AstExpr<'a>>),

    /// An identifier
    Identifier(String),

    /// A binary operation between two expressions
    BinaryOp { left: Box<AstExpr<'a>>, operation: Operation, right: Box<AstExpr<'a>> },

    /// Address of a variable
    AddressOf(String),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<AstExpr<'a>>),

    /// Function call
    FunctionCall { function: Box<AstExpr<'a>>, arguments: Vec<AstExpr<'a>> },

    /// Index expression
    IndexAccess { left: Box<AstExpr<'a>>, index: Box<AstExpr<'a>> },

    /// Type cast
    TypeCast { new_type: UnresolvedType, expr: Box<AstExpr<'a>> },

    /// An if statement
    IfExpression {
        condition: Box<AstExpr<'a>>,
        body: Box<AstBlock<'a>>,
    },

    /// A while loop
    WhileLoop {
        condition: Box<AstExpr<'a>>,
        body: Box<AstBlock<'a>>
    },

    /// For loop
    ForLoop {
        init: Box<AstStmt<'a>>,
        condition: Box<AstExpr<'a>>,
        update: Box<AstStmt<'a>>,
        body: Box<AstBlock<'a>>
    },

    /// Unconditional loop
    Loop {
        body: Box<AstBlock<'a>>
    },

    /// A Block expression
    BlockExpression(Box<AstBlock<'a>>),
}


impl<'a> Debug for AstExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for AstExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstExprKind::Number(n) => write!(f, "{}", n),
            AstExprKind::String(s) => write!(f, "{}", s),
            AstExprKind::Char(c) => write!(f, "{}", c),
            AstExprKind::Bool(b) => write!(f, "{}", b),
            AstExprKind::ArrayLiteral(items) => {
                write!(f, "[")?;
                write!(f, "{:?}", items)?;
                write!(f, "]")
            }
            AstExprKind::Identifier(i) => write!(f, "{}", i),
            AstExprKind::BinaryOp { left, operation, right } => {
                write!(f, "({} {} {})", left, operation, right)
            },
            AstExprKind::AddressOf(i) => write!(f, "{}", i),
            AstExprKind::Dereference(e) => write!(f, "*{}", e),
            AstExprKind::FunctionCall { function, arguments } => {
                write!(f, "{}({:?})", function, arguments)
            },
            AstExprKind::IndexAccess { left, index } => {
                write!(f, "{}[{}]", left, index)
            },
            AstExprKind::TypeCast {new_type, expr } => {
                write!(f, "{} as {}", expr, new_type)
            },
            AstExprKind::IfExpression { condition, body } => {
                write!(f, "if {} {}", condition, body)
            },
            AstExprKind::WhileLoop { condition, body } => {
                write!(f, "while {} {}", condition, body)
            },
            AstExprKind::ForLoop { init, condition, update, body } => {
                write!(f, "for {} {} {} {}", init, condition, update, body)
            },
            AstExprKind::Loop { body } => {
                write!(f, "loop {}", body)
            },
            AstExprKind::BlockExpression(block) => {
                write!(f, "{}", block)
            }
        }
    }
}