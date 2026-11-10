use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::thir::expr::ThirExpr;
use crate::compiler::ir::thir::item::ThirItem;
use crate::compiler::ir::types::Type;
use crate::compiler::ir::types::type_table::TypeTable;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ThirStmt<'a> {
    pub kind: ThirStmtKind<'a>,
    pub span: Span<'a>,
}

impl<'a> ThirStmt<'a> {
    pub fn new(span: Span<'a>, kind: ThirStmtKind<'a>) -> ThirStmt<'a> {
        ThirStmt { kind, span }
    }
}

impl<'a> ThirStmt<'a> {
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
        }
    }
}


#[derive(Debug, Clone)]
pub enum ThirStmtKind<'a> {
    /// An assignment of an expression to some expression
    Assignment {
        target: Box<ThirExpr<'a>>,
        value: Box<ThirExpr<'a>>,
        ty: Type
    },

    /// A variable declaration
    VariableDeclaration {
        symbol_id: SymbolId,
        value: Option<Box<ThirExpr<'a>>>,
        ty: Type,
    },

    /// Return
    Return { value: Option<Box<ThirExpr<'a>>>, ty: Type },

    /// An item like a function declaration
    Item(Box<ThirItem<'a>>),

    /// Expression used as a statement, with no trailing semicolon, for when a block evaluates to an expr
    Expression(Box<ThirExpr<'a>>),

    /// Expression statement with trailing semicolon, for stuff like function calls outside of statements
    Semi(Box<ThirExpr<'a>>),
}