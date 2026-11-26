use crate::compiler::errors::Span;
use crate::compiler::ir::ast::{AstBlock, AstStmt};
use crate::compiler::ir::operations::*;
use crate::compiler::ir::types::written_type::*;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub struct AstExpr {
    pub kind: AstExprKind,
    pub span: Span,
}

impl AstExpr {
    pub fn new(span: Span, kind: AstExprKind) -> AstExpr {
        AstExpr { kind, span }
    }
}

#[derive(Clone, Debug)]
pub enum AstExprKind {
    /// Number
    Number(i32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<AstExpr>),

    /// An identifier
    Identifier(String),

    /// A binary operation between two expressions
    BinaryOp { left: Box<AstExpr>, operation: Operation, right: Box<AstExpr> },

    /// A unary operation
    UnaryOp { operation: UnaryOperation, right: Box<AstExpr> },

    /// Address of a variable
    AddressOf(Box<AstExpr>),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<AstExpr>),

    /// Struct literal like MyStruct { a: 10 }
    StructLiteral {
        name: String,
        fields: Vec<AstFieldExpr>
    },

    /// Member access like strooct.thing
    MemberAccess {
        left: Box<AstExpr>,
        member: String,
    },

    /// Dereferenced member access, e.g. a->b
    DerefMemberAccess {
        left: Box<AstExpr>,
        member: String,
    },

    /// Function call
    FunctionCall { function: Box<AstExpr>, arguments: Vec<AstExpr> },

    /// Index expression
    IndexAccess { left: Box<AstExpr>, index: Box<AstExpr> },

    /// Type cast
    TypeCast { new_type: WrittenType, expr: Box<AstExpr> },

    /// Continue
    Continue { value: Option<Box<AstExpr>> },

    /// Break
    Break { value: Option<Box<AstExpr>> },

    /// An if statement
    IfExpression {
        condition: Box<AstExpr>,
        body: Box<AstBlock>,

        /// Optional else condition, else block
        else_statements: Vec< (Option<Box<AstExpr>>, Box<AstBlock>) >,
    },

    /// A while loop
    WhileLoop {
        condition: Box<AstExpr>,
        body: Box<AstBlock>
    },

    /// For loop
    ForLoop {
        init: Box<AstStmt>,
        condition: Box<AstExpr>,
        update: Box<AstStmt>,
        body: Box<AstBlock>
    },

    /// Unconditional loop
    Loop {
        body: Box<AstBlock>
    },

    /// A Block expression
    BlockExpression(Box<AstBlock>),
}


impl Debug for AstExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for AstExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstExprKind::Number(n) => write!(f, "{}", n),
            AstExprKind::String(s) => write!(f, "{}", s),
            AstExprKind::Char(c) => write!(f, "{}", c),
            AstExprKind::Bool(b) => write!(f, "{}", b),
            AstExprKind::ArrayLiteral(items) => {
                write!(f, "[")?;
                write!(f, "{:?}", items)?;
                write!(f, "]")
            }
            AstExprKind::Identifier(i) => write!(f, "{}", i),
            AstExprKind::BinaryOp { left, operation, right } => {
                write!(f, "({} {} {})", left, operation, right)
            },
            AstExprKind::UnaryOp { operation, right } => {
                write!(f, "({}{})", operation, right)
            },
            AstExprKind::AddressOf(i) => write!(f, "{}", i),
            AstExprKind::Dereference(e) => write!(f, "*{}", e),
            AstExprKind::StructLiteral { name,  fields } => {
                write!(f, "{} {{", name)?;
                write!(f, "{}", fields.iter().map(|e| { e.to_string() }).collect::<String>())?;
                write!(f, "}}")
            },
            AstExprKind::MemberAccess { left, member } => {
                write!(f, "({}.{})", left, member)
            },
            AstExprKind::DerefMemberAccess { left, member } => {
                write!(f, "({}->{})", left, member)
            }
            AstExprKind::FunctionCall { function, arguments } => {
                write!(f, "{}({:?})", function, arguments)
            },
            AstExprKind::IndexAccess { left, index } => {
                write!(f, "{}[{}]", left, index)
            },
            AstExprKind::TypeCast {new_type, expr } => {
                write!(f, "{} as {}", expr, new_type.kind)
            },
            AstExprKind::Continue { value } => {
                if let Some(value) = value {
                    write!(f, "continue {}", value)
                } else {
                    write!(f, "continue")
                }
            },
            AstExprKind::Break { value } => {
                if let Some(value) = value {
                    write!(f, "break {}", value)
                } else {
                    write!(f, "break")
                }
            },
            AstExprKind::IfExpression { condition, body, else_statements } => {
                write!(f, "if {} {}", condition, body)?;
                for el in else_statements {
                    write!(f, " else if {:?} {}", el.0, el.1)?;
                };
                write!(f, " end")
            },
            AstExprKind::WhileLoop { condition, body } => {
                write!(f, "while {} {}", condition, body)
            },
            AstExprKind::ForLoop { init, condition, update, body } => {
                write!(f, "for {} {} {} {}", init, condition, update, body)
            },
            AstExprKind::Loop { body } => {
                write!(f, "loop {}", body)
            },
            AstExprKind::BlockExpression(block) => {
                write!(f, "{}", block)
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct AstFieldExpr {
    pub name: String,
    pub value: Box<AstExpr>,
    pub span: Span,
}

impl Display for AstFieldExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}