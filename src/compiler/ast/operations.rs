use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum Operation {
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

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operation::Add => write!(f, "+"),
            Operation::Subtract => write!(f, "-"),
            Operation::Multiply => write!(f, "*"),
            Operation::Divide => write!(f, "/"),
            Operation::Modulo => write!(f, "%"),
            Operation::GreaterThan => write!(f, ">"),
            Operation::LessThan => write!(f, "<"),
            Operation::GreaterThanOrEqual => write!(f, ">="),
            Operation::LessThanOrEqual => write!(f, "<="),
            Operation::Equal => write!(f, "=="),
            Operation::NotEqual => write!(f, "!="),
            Operation::Or => write!(f, "||"),
            Operation::And => write!(f, "&&"),
            Operation::Xor => write!(f, "^"),
        }
    }
}