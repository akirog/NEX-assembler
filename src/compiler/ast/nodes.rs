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
pub enum AstNodeKind {
    /// Any value that can be represented in 4 bytes
    Value(u32),

    /// An identifier
    Ident(String),

    /// A binary operation between two expressions
    BinaryOp { left: Box<AstNodeKind>, operation: AstNodeOperation, right: Box<AstNodeKind> },
    
    /// An assignment of an expression to some expression
    Assignment { target: Box<AstNodeKind>, value: Box<AstNodeKind> },
}


#[derive(Debug)]
pub struct AstNode<'a> {
    pub span: Span<'a>,
    pub kind: AstNodeKind,
}

impl<'a> AstNode<'a> {
    pub fn new(span: Span<'a>, kind: AstNodeKind) -> AstNode<'a> {
        AstNode { span, kind }
    }
}