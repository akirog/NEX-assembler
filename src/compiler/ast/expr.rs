use std::fmt::{Debug, Display, Formatter};
use compiler::ast::operations::Operation;
use compiler::ast::ValueType;
use compiler::errors::Span;


pub struct ExprNode<'a> {
    pub kind: ExprKind<'a>,
    pub span: Span<'a>,
    pub value_type: Option<ValueType>,
}

impl<'a> ExprNode<'a> {
    pub fn new(span: Span<'a>, kind: ExprKind<'a>,  value_type: Option<ValueType>) -> ExprNode<'a> {
        ExprNode { kind, span, value_type}
    }
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    /// Number
    Number(u32),

    /// An identifier
    Identifier(String),

    /// A binary operation between two expressions
    BinaryOp { left: Box<ExprNode<'a>>, operation: Operation, right: Box<ExprNode<'a>> },

    /// Address of a variable
    AddressOf(String),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<ExprNode<'a>>),

    /// Function call
    FunctionCall { function: Box<ExprNode<'a>>, arguments: Vec<ExprNode<'a>> },
}


impl<'a> Debug for ExprNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for ExprNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ExprKind::Number(n) => write!(f, "{}", n),
            ExprKind::Identifier(i) => write!(f, "{}", i),
            ExprKind::BinaryOp { left, operation, right } => {
                write!(f, "({} {} {})", left, operation, right)
            },
            ExprKind::AddressOf(i) => write!(f, "{}", i),
            ExprKind::Dereference(e) => write!(f, "{}", e),
            ExprKind::FunctionCall { function, arguments } => {
                write!(f, "{}({:?})", function, arguments)
            }
        }
    }
}