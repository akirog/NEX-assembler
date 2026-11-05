use std::fmt::{Debug, Display};

#[derive(Clone, Debug)]
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

impl Operation {
    pub fn precedence(&self) -> usize {
        match self {
            Operation::Add | Operation::Subtract => 1,
            Operation::Multiply | Operation::Divide | Operation::Modulo => 2,
            Operation::GreaterThan | Operation::LessThan => 3,
            Operation::GreaterThanOrEqual | Operation::LessThanOrEqual => 4,
            Operation::Equal | Operation::NotEqual => 5,
            Operation::Or | Operation::And | Operation::Xor => 6,
        }
    }

    pub fn assembly_text(&self) -> String {
        match self {
            Operation::Add => "add".to_string(),
            Operation::Subtract => "sub".to_string(),
            Operation::Multiply => "mul".to_string(),
            Operation::Divide => "div".to_string(),
            Operation::Modulo => "mod".to_string(),
            Operation::GreaterThan => "gt".to_string(),
            Operation::LessThan => "lt".to_string(),
            Operation::GreaterThanOrEqual => "ge".to_string(),
            Operation::LessThanOrEqual => "le".to_string(),
            Operation::Equal => "eq".to_string(),
            Operation::NotEqual => "ne".to_string(),
            Operation::Or => "or".to_string(),
            Operation::And => "and".to_string(),
            Operation::Xor => "xor".to_string(),
        }
    }
}