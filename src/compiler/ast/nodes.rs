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
pub enum AstNodeKind<'a, Ty: Debug> {
    /// Number\
    /// value, type
    Number(u32, Ty),

    /// An identifier\
    /// name, type
    Identifier(String, Ty),

    /// A binary operation between two expressions
    BinaryOp { left: Box<AstNode<'a, Ty>>, operation: AstNodeOperation, right: Box<AstNode<'a, Ty>>, result_type: Ty },

    /// An assignment of an expression to some expression
    Assignment { target: Box<AstNode<'a, Ty>>, value: Box<AstNode<'a, Ty>>, target_type: Ty },
    
    /// A variable declaration
    Declaration { var_type: Ty, name: String, value: Box<AstNode<'a, Ty>> },

    /// Address of a variable
    AddressOf(String),

    /// Memory dereference of a value, e.g. `*ptr`\
    /// value, result_type
    Dereference(Box<AstNode<'a, Ty>>, Ty),
}


pub struct AstNode<'a, Ty: Debug> {
    pub span: Span<'a>,
    pub kind: AstNodeKind<'a, Ty>,
}


impl<'a, Ty: Debug> AstNode<'a, Ty> {
    pub fn new(span: Span<'a>, kind: AstNodeKind<'a, Ty>) -> AstNode<'a, Ty> {
        AstNode { span, kind }
    }
}

impl<'a, Ty: Debug> Debug for AstNode<'a, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a, Ty: Debug> Display for AstNode<'a, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstNodeKind::Number(n, _) => write!(f, "{}", n),
            AstNodeKind::Identifier(name, _) => write!(f, "{}", name),
            AstNodeKind::BinaryOp { left, operation, right, .. } => {
                write!(f, "{} {} {}", left, operation, right)
            },
            AstNodeKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            AstNodeKind::Declaration { var_type, name, value } => {
                write!(f, "{:?} {} = {}", var_type, name, value)
            },
            AstNodeKind::AddressOf(address) => write!(f, "&{}", address),
            AstNodeKind::Dereference(deref, _) => write!(f, "*{}", deref),
        }
    }
}