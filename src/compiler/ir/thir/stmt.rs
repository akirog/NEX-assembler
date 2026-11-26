use crate::compiler::assembly_generation::register_manager::Register;
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::thir::expr::ThirExpr;
use crate::compiler::ir::thir::item::ThirItem;
use crate::compiler::ir::types::Type;
use crate::compiler::ir::types::type_table::TypeTable;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ThirStmt {
    pub kind: ThirStmtKind,
    pub span: Span,
}

impl ThirStmt {
    pub fn new(span: Span, kind: ThirStmtKind) -> ThirStmt {
        ThirStmt { kind, span }
    }
}

impl ThirStmt {
    pub fn pretty_print(&self, type_table: &TypeTable) -> String {
        match &self.kind {
            ThirStmtKind::VariableDeclaration { symbol_id, value, ty } => {
                if let Some(value) = value {
                    format!("let {}: {} = {};", symbol_id, type_table.type_as_string(&ty.id), value.pretty_print(type_table))
                } else {
                    format!("let {}: {};", symbol_id, type_table.type_as_string(&ty.id))
                }
            },
            ThirStmtKind::Assignment { target, value, .. } => {
                format!("{} = {}", target.pretty_print(type_table), value.pretty_print(type_table))
            },
            ThirStmtKind::Return { value, .. } => {
                if let Some(returned_value) = value {
                    format!("return {}", returned_value.pretty_print(type_table))
                } else {
                    "return".to_string()
                }
            },
            ThirStmtKind::Item(item) => {
                format!("{}", item.pretty_print(type_table))
            },
            ThirStmtKind::Expression(expression) => {
                format!("{}", expression.pretty_print(type_table))
            },
            ThirStmtKind::Semi(expression) => {
                format!("{};", expression.pretty_print(type_table))
            },
            ThirStmtKind::AsmBlock { inputs, outputs, lines } => {
                let mut string = Vec::new();
                string.push(format!("asm block {{\n\tinputs:"));
                string.push("\tinputs:".to_string());
                for input in inputs {
                    string.push(format!("\t\t{} to {}", input.0.pretty_print(type_table), input.1));
                }
                string.push("\toutputs:".to_string());
                for output in outputs {
                    string.push(format!("\t\t{} to {}", output.1, output.0.pretty_print(type_table)));
                }
                string.extend(lines.clone());
                string.push(format!("}}"));
                string.join("\n")
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum ThirStmtKind {
    /// An assignment of an expression to some expression
    Assignment {
        target: Box<ThirExpr>,
        value: Box<ThirExpr>,
        ty: Type
    },

    /// A variable declaration
    VariableDeclaration {
        symbol_id: SymbolId,
        value: Option<Box<ThirExpr>>,
        ty: Type,
    },

    /// Return
    Return { value: Option<Box<ThirExpr>>, ty: Type },

    /// An item like a function declaration
    Item(Box<ThirItem>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<ThirExpr>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<ThirExpr>),

    /// Assembly block
    AsmBlock {
        inputs: Vec<(ThirExpr, Register)>,
        outputs: Vec<(ThirExpr, Register)>,
        lines: Vec<String>
    }
}