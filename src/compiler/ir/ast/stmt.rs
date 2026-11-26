use crate::compiler::assembly_generation::register_manager::Register;
use crate::compiler::errors::Span;
use crate::compiler::ir::ast::expr::AstExpr;
use crate::compiler::ir::ast::item::AstItem;
use crate::compiler::ir::types::written_type::*;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub struct AstStmt {
    pub kind: AstStmtKind,
    pub span: Span,
}

impl AstStmt {
    pub fn new(span: Span, kind: AstStmtKind) -> AstStmt {
        AstStmt { kind, span }
    }
}

impl Debug for AstStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for AstStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstStmtKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            AstStmtKind::VariableDeclaration { var_type, name, value } => {
                write!(f, "let {}: {:?} = {:?}", name, var_type, value)
            },
            AstStmtKind::Return { value } => {
                write!(f, "return {:?}", value)
            },
            AstStmtKind::Item(item) => {
                write!(f, "item {}", item)
            },
            AstStmtKind::Expression(expr) => {
                write!(f, "expr {}", expr)
            },
            AstStmtKind::Semi(expression ) => {
                write!(f, "{};", expression)
            },
            AstStmtKind::AsmBlock { inputs, outputs, lines } => {
                write!(f, "asm block {{")?;
                write!(f, " inputs:")?;
                for input in inputs {
                    write!(f, "{} to {}", input.0, input.1)?;
                }
                write!(f, " outputs:")?;
                for output in outputs {
                    write!(f, "{} to {}", output.1, output.0)?;
                }
                for line in lines {
                    write!(f, "\t\t{}", line)?;
                }
                write!(f, " }}")
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum AstStmtKind {
    /// An assignment of an expression to some expression
    Assignment {
        target: Box<AstExpr>,
        value: Box<AstExpr>
    },

    /// A variable declaration
    VariableDeclaration {
        name: String,
        value: Option<Box<AstExpr>>,
        var_type: Option<WrittenType>,
    },

    /// Return
    Return {
        value: Option<Box<AstExpr>>,
    },

    /// An item like a function declaration or struct definition
    Item(Box<AstItem>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<AstExpr>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<AstExpr>),

    /// Assembly block
    AsmBlock {
        inputs: Vec<(AstExpr, Register)>,
        outputs: Vec<(AstExpr, Register)>,
        lines: Vec<String>
    }
}