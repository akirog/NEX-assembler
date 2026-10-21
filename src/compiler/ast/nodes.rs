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

impl Display for AstNodeOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AstNodeOperation::Add => write!(f, "+"),
            AstNodeOperation::Subtract => write!(f, "-"),
            AstNodeOperation::Multiply => write!(f, "*"),
            AstNodeOperation::Divide => write!(f, "/"),
            AstNodeOperation::Modulo => write!(f, "%"),
            AstNodeOperation::GreaterThan => write!(f, ">"),
            AstNodeOperation::LessThan => write!(f, "<"),
            AstNodeOperation::GreaterThanOrEqual => write!(f, ">="),
            AstNodeOperation::LessThanOrEqual => write!(f, "<="),
            AstNodeOperation::Equal => write!(f, "=="),
            AstNodeOperation::NotEqual => write!(f, "!="),
            AstNodeOperation::Or => write!(f, "||"),
            AstNodeOperation::And => write!(f, "&&"),
            AstNodeOperation::Xor => write!(f, "^"),
        }
    }
}


#[derive(Debug)]
pub enum AstNodeKind<'a> {
    /// Number
    Number(u32),

    /// Boolean
    Boolean(bool),

    /// Character
    Char(char),

    /// An identifier
    Identifier(String),

    /// A binary operation between two expressions
    BinaryOp { left: Box<AstNode<'a>>, operation: AstNodeOperation, right: Box<AstNode<'a>> },

    /// An assignment of an expression to some expression
    Assignment { target: Box<AstNode<'a>>, value: Box<AstNode<'a>> },
    
    /// A variable declaration
    Declaration { var_type: String, name: String, value: Box<AstNode<'a>> },

    /// Address of a variable
    AddressOf(String),
    
    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<AstNode<'a>>),
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

impl<'a> Display for AstNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstNodeKind::Number(n) => write!(f, "{}", n),
            AstNodeKind::Boolean(b) => write!(f, "{}", b),
            AstNodeKind::Char(c) => write!(f, "{}", c),
            AstNodeKind::Identifier(name) => write!(f, "{}", name),
            AstNodeKind::BinaryOp { left, operation, right } => {
                write!(f, "{} {} {}", left, operation, right)
            },
            AstNodeKind::Assignment { target, value } => {
                write!(f, "{} = {}", target, value)
            },
            AstNodeKind::Declaration { var_type, name, value } => {
                write!(f, "{} {} = {}", var_type, name, value)
            },
            AstNodeKind::AddressOf(address) => write!(f, "&{}", address),
            AstNodeKind::Dereference(deref) => write!(f, "*{}", deref),
        }
    }
}