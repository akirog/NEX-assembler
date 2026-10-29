use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::{HirBlock, HirStmtKind};
use crate::compiler::ir::operations::*;
use crate::compiler::ir::types::unresolved_type::*;


#[derive(Debug, Clone)]
pub struct SymbolId(uid::Id<u16>);

#[derive(Clone)]
pub struct HirExpr<'a> {
    pub kind: HirExprKind<'a>,
    pub span: Span<'a>,
}

impl<'a> HirExpr<'a> {
    pub fn new(span: Span<'a>, kind: HirExprKind<'a>) -> HirExpr<'a> {
        HirExpr { kind, span }
    }
}

#[derive(Clone, Debug)]
pub enum HirExprKind<'a> {
    /// Number
    Number(u32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<HirExpr<'a>>),

    /// An identifier
    Identifier(SymbolId),

    /// A binary operation between two expressions
    BinaryOp { left: Box<HirExpr<'a>>, operation: Operation, right: Box<HirExpr<'a>> },

    /// Address of a variable
    AddressOf(String),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<HirExpr<'a>>),

    /// Function call
    FunctionCall { function: Box<HirExpr<'a>>, arguments: Vec<HirExpr<'a>> },

    /// Index expression
    IndexAccess { left: Box<HirExpr<'a>>, index: Box<HirExpr<'a>> },

    /// Type cast
    TypeCast { new_type: UnresolvedType, expr: Box<HirExpr<'a>> },

    /// An if statement
    IfExpression {
        condition: Box<HirExpr<'a>>,
        body: Box<HirBlock<'a>>,
    },

    /// A Block expression
    BlockExpression(Box<HirBlock<'a>>),
}


impl<'a> Debug for HirExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for HirExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HirExprKind::Number(n) => write!(f, "{}", n),
            HirExprKind::String(s) => write!(f, "{}", s),
            HirExprKind::Char(c) => write!(f, "{}", c),
            HirExprKind::Bool(b) => write!(f, "{}", b),
            HirExprKind::ArrayLiteral(items) => {
                write!(f, "[")?;
                write!(f, "{:?}", items)?;
                write!(f, "]")
            }
            HirExprKind::Identifier(i) => write!(f, "{:?}", i),
            HirExprKind::BinaryOp { left, operation, right } => {
                write!(f, "({} {} {})", left, operation, right)
            },
            HirExprKind::AddressOf(i) => write!(f, "{}", i),
            HirExprKind::Dereference(e) => write!(f, "*{}", e),
            HirExprKind::FunctionCall { function, arguments } => {
                write!(f, "{}({:?})", function, arguments)
            },
            HirExprKind::IndexAccess { left, index } => {
                write!(f, "{}[{}]", left, index)
            },
            HirExprKind::TypeCast {new_type, expr } => {
                write!(f, "{} as {}", expr, new_type)
            },
            HirExprKind::IfExpression { condition, body } => {
                write!(f, "if {} {}", condition, body)
            },
            HirExprKind::BlockExpression(block) => {
                write!(f, "{}", block)
            }
        }
    }
}