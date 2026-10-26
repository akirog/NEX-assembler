use std::fmt::{Debug, Display, Formatter};
use crate::compiler::*;


#[derive(Clone)]
pub struct ExprNode<'a> {
    pub kind: ExprKind<'a>,
    pub span: Span<'a>,
    pub resolved_type: Option<ResolvedType>,
}

impl<'a> ExprNode<'a> {
    pub fn new(span: Span<'a>, kind: ExprKind<'a>) -> ExprNode<'a> {
        ExprNode { kind, span, resolved_type: None }
    }
}

#[derive(Clone, Debug)]
pub enum ExprKind<'a> {
    /// Number
    Number(u32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<ExprNode<'a>>),

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

    /// Index expression
    IndexAccess { left: Box<ExprNode<'a>>, index: Box<ExprNode<'a>> },

    /// Type cast
    TypeCast { new_type: UnresolvedType, expr: Box<ExprNode<'a>> },
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
            ExprKind::String(s) => write!(f, "{}", s),
            ExprKind::Char(c) => write!(f, "{}", c),
            ExprKind::Bool(b) => write!(f, "{}", b),
            ExprKind::ArrayLiteral(items) => {
                write!(f, "[")?;
                write!(f, "{:?}", items)?;
                write!(f, "]")
            }
            ExprKind::Identifier(i) => write!(f, "{}", i),
            ExprKind::BinaryOp { left, operation, right } => {
                write!(f, "({} {} {})", left, operation, right)
            },
            ExprKind::AddressOf(i) => write!(f, "{}", i),
            ExprKind::Dereference(e) => write!(f, "*{}", e),
            ExprKind::FunctionCall { function, arguments } => {
                write!(f, "{}({:?})", function, arguments)
            },
            ExprKind::IndexAccess { left, index } => {
                write!(f, "{}[{}]", left, index)
            },
            ExprKind::TypeCast {new_type, expr } => {
                write!(f, "{} as {}", expr, new_type)
            },
        }
    }
}