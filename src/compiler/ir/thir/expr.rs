use std::fmt::{Debug, Display, Formatter};
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::thir::{ThirBlock, ThirStmtKind};
use crate::compiler::ir::operations::*;
use crate::compiler::ir::types::unresolved_type::*;



#[derive(Clone)]
pub struct ThirExpr<'a> {
    pub kind: ThirExprKind<'a>,
    pub span: Span<'a>,
}

impl<'a> ThirExpr<'a> {
    pub fn new(span: Span<'a>, kind: ThirExprKind<'a>) -> ThirExpr<'a> {
        ThirExpr { kind, span }
    }
}

#[derive(Clone, Debug)]
pub enum ThirExprKind<'a> {
    /// Number
    Number(i32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<ThirExpr<'a>>),

    /// An identifier
    Identifier(SymbolId),

    /// A binary operation between two expressions
    BinaryOp { left: Box<ThirExpr<'a>>, operation: Operation, right: Box<ThirExpr<'a>> },

    /// Address of a variable
    AddressOf(SymbolId),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<ThirExpr<'a>>),

    /// Function call
    FunctionCall { function: Box<ThirExpr<'a>>, arguments: Vec<ThirExpr<'a>> },

    /// Index expression
    IndexAccess { left: Box<ThirExpr<'a>>, index: Box<ThirExpr<'a>> },

    /// Type cast
    TypeCast { new_type: UnresolvedType, expr: Box<ThirExpr<'a>> },

    /// Break
    Break { value: Option<Box<ThirExpr<'a>>> },

    /// Continue
    Continue { value: Option<Box<ThirExpr<'a>>> },

    /// An if statement
    IfExpression {
        condition: Box<ThirExpr<'a>>,
        body: Box<ThirBlock<'a>>,

        /// Optional else condition, else block
        else_statements: Vec< (Option<Box<ThirExpr<'a>>>, Box<ThirBlock<'a>>) >,
    },

    /// Unconditional loop
    Loop {
        body: Box<ThirBlock<'a>>,
    },

    /// A Block expression
    BlockExpression(Box<ThirBlock<'a>>),
}


impl<'a> Debug for ThirExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for ThirExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ThirExprKind::Number(n) => write!(f, "{}", n),
            ThirExprKind::String(s) => write!(f, "{}", s),
            ThirExprKind::Char(c) => write!(f, "{}", c),
            ThirExprKind::Bool(b) => write!(f, "{}", b),
            ThirExprKind::ArrayLiteral(items) => {
                write!(f, "[")?;
                write!(f, "{:?}", items)?;
                write!(f, "]")
            }
            ThirExprKind::Identifier(i) => write!(f, "{}", i),
            ThirExprKind::BinaryOp { left, operation, right } => {
                write!(f, "({} {} {})", left, operation, right)
            },
            ThirExprKind::AddressOf(i) => write!(f, "{:?}", i),
            ThirExprKind::Dereference(e) => write!(f, "*{}", e),
            ThirExprKind::FunctionCall { function, arguments } => {
                write!(f, "{}({:?})", function, arguments)
            },
            ThirExprKind::IndexAccess { left, index } => {
                write!(f, "{}[{}]", left, index)
            },
            ThirExprKind::TypeCast {new_type, expr } => {
                write!(f, "{} as {}", expr, new_type)
            },
            ThirExprKind::Break { value } => {
                if let Some(value) = value {
                    write!(f, "break {}", value)
                } else {
                    write!(f, "continue")
                }
            },
            ThirExprKind::Continue { value } => {
                if let Some(value) = value {
                    write!(f, "continue {}", value)
                } else {
                    write!(f, "continue")
                }
            },
            ThirExprKind::IfExpression { condition, body, else_statements } => {
                write!(f, "if {} {}", condition, body)?;
                for el in else_statements {
                    write!(f, " else if {:?} {}", el.0, el.1)?;
                };
                write!(f, " end")
            },
            ThirExprKind::Loop { body } => {
                write!(f, "loop {}", body)
            },
            ThirExprKind::BlockExpression(block) => {
                write!(f, "{}", block)
            }
        }
    }
}