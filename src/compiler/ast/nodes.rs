use std::fmt::{Debug, Display, Formatter};
use compiler::errors::{Span};


#[derive(Debug)]
pub enum AstNodeOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
    Or,
    And,
    Xor,
}

#[derive(Debug)]
pub enum AstNodeKind<'a> {
    /// Any value that can be represented in 4 bytes
    Value(u32),

    /// An identifier
    Ident(String),

    /// A binary operation between two expressions
    BinaryOp { left: Box<AstNode<'a>>, operation: AstNodeOperation, right: Box<AstNode<'a>> },

    /// An assignment of an expression to some expression
    Assignment { target: Box<AstNode<'a>>, value: Box<AstNode<'a>> },
}


pub struct AstNode<'a> {
    pub span: Span<'a>,
    pub kind: AstNodeKind<'a>,
}

impl<'a> AstNode<'a> {
    pub fn new(span: Span<'a>, kind: AstNodeKind<'a>) -> AstNode<'a> {
        AstNode { span, kind }
    }
}

impl<'a> Debug for AstNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}