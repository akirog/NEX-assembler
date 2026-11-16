use crate::compiler::assembly_generation::register_manager::Register;
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::hir::expr::HirExpr;
use crate::compiler::ir::hir::item::HirItem;
use crate::compiler::ir::types::written_type::*;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub struct HirStmt<'a> {
    pub kind: HirStmtKind<'a>,
    pub span: Span<'a>,
}

impl<'a> HirStmt<'a> {
    pub fn new(span: Span<'a>, kind: HirStmtKind<'a>) -> HirStmt<'a> {
        HirStmt { kind, span }
    }
}

impl<'a> Debug for HirStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for HirStmt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HirStmtKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            HirStmtKind::VariableDeclaration { var_type, symbol_id, value } => {
                write!(f, "let {}: {:?} = {:?}", symbol_id, var_type, value)
            },
            HirStmtKind::Return { value } => {
                write!(f, "return {:?}", value)
            },
            HirStmtKind::Item(item) => {
                write!(f, "item {}", item)
            },
            HirStmtKind::Expression(expr) => {
                write!(f, "expr {}", expr)
            }
            HirStmtKind::Semi(expression ) => {
                write!(f, "{};", expression)
            },
            HirStmtKind::AsmBlock { inputs, outputs, lines } => {
                write!(f, "asm block {{")?;
                write!(f, "\tinputs:")?;
                for input in inputs {
                    write!(f, "\t\t{} to {}", input.0, input.1)?;
                }
                write!(f, "\toutputs:")?;
                for output in outputs {
                    write!(f, "\t\t{} to {}", output.1, output.0)?;
                }
                for line in lines {
                    write!(f, "\t{}", line)?;
                }
                write!(f, " }}")
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum HirStmtKind<'a> {
    /// An assignment of an expression to some expression
    Assignment {
        target: Box<HirExpr<'a>>,
        value: Box<HirExpr<'a>>
    },

    /// A variable declaration
    VariableDeclaration {
        symbol_id: SymbolId,
        value: Option<Box<HirExpr<'a>>>,
        var_type: Option<WrittenType<'a>>,
    },

    /// Return
    Return { value: Option<Box<HirExpr<'a>>> },

    /// An item like a function declaration
    Item(Box<HirItem<'a>>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<HirExpr<'a>>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<HirExpr<'a>>),

    /// Assembly block
    AsmBlock {
        inputs: Vec<(HirExpr<'a>, Register)>,
        outputs: Vec<(HirExpr<'a>, Register)>,
        lines: Vec<String>
    }
}