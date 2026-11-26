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
    LogicalAnd,
    LogicalOr,
    SHL,
    SHR,
    Or,
    And,
    Xor,
}

impl Operation {
    pub fn try_imm(&self) -> Option<ImmOperation> {
        let op = match self {
            Operation::Add => ImmOperation::Add,
            Operation::Subtract => ImmOperation::Subtract,
            Operation::Multiply => ImmOperation::Multiply,
            Operation::Divide => ImmOperation::Divide,
            Operation::SHL => ImmOperation::SHL,
            Operation::SHR => ImmOperation::SHR,
            _ => return None
        };

        Some(op)
    }
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
            Operation::SHL => write!(f, "<<"),
            Operation::SHR => write!(f, ">>"),
            Operation::Or => write!(f, "|"),
            Operation::And => write!(f, "&"),
            Operation::LogicalAnd => write!(f, "&&"),
            Operation::LogicalOr => write!(f, "||"),
            Operation::Xor => write!(f, "^"),
        }
    }
}

impl Operation {
    pub fn precedence(&self) -> usize {
        match self {
            Operation::LogicalOr => 1,
            Operation::LogicalAnd => 2,

            Operation::Or => 3,
            Operation::Xor => 4,
            Operation::And => 5,

            Operation::Equal | Operation::NotEqual => 6,

            Operation::GreaterThan | Operation::LessThan |
            Operation::GreaterThanOrEqual | Operation::LessThanOrEqual => 7,

            Operation::SHL | Operation::SHR => 8,

            Operation::Add | Operation::Subtract => 9,

            Operation::Multiply | Operation::Divide | Operation::Modulo => 10,
        }
    }

    pub fn assembly_text(&self) -> String {
        match self {
            Operation::Add => "add",
            Operation::Subtract => "sub",
            Operation::Multiply => "mul",
            Operation::Divide => "div",
            Operation::Modulo => "mod",
            Operation::GreaterThan => "gt",
            Operation::LessThan => "lt",
            Operation::GreaterThanOrEqual => "ge",
            Operation::LessThanOrEqual => "le",
            Operation::Equal => "eq",
            Operation::NotEqual => "ne",
            Operation::SHL => "shl",
            Operation::SHR => "shr",
            Operation::Or => "or",
            Operation::And => "and",
            Operation::LogicalAnd => "and",
            Operation::LogicalOr => "or",
            Operation::Xor => "xor",
        }.to_string()
    }
}


#[derive(Clone, Debug)]
pub enum UnaryOperation {
    Negate,
    Not,
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnaryOperation::Negate => write!(f, "-"),
            UnaryOperation::Not => write!(f, "!"),
        }
    }
}

impl UnaryOperation {
    pub fn assembly_text(&self) -> String {
        match self {
            UnaryOperation::Negate => "neg",
            UnaryOperation::Not => "not",
        }.to_string()
    }
}


#[derive(Clone, Debug)]
pub enum ImmOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    SHL,
    SHR,
}

impl Display for ImmOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImmOperation::Add => write!(f, "+"),
            ImmOperation::Subtract => write!(f, "-"),
            ImmOperation::Multiply => write!(f, "*"),
            ImmOperation::Divide => write!(f, "/"),
            ImmOperation::SHL => write!(f, "<<"),
            ImmOperation::SHR => write!(f, ">>"),
        }
    }
}

impl ImmOperation {
    pub fn assembly_text(&self) -> String {
        match self {
            ImmOperation::Add => "addi".to_string(),
            ImmOperation::Subtract => "subi".to_string(),
            ImmOperation::Multiply => "muli".to_string(),
            ImmOperation::Divide => "divi".to_string(),
            ImmOperation::SHL => "shli".to_string(),
            ImmOperation::SHR => "shri".to_string(),
        }
    }
}