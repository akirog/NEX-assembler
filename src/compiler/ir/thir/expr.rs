use std::collections::HashMap;
use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::operations::*;
use crate::compiler::ir::thir::ThirBlock;
use crate::compiler::ir::types::resolved_type::*;
use crate::compiler::ir::types::type_table::TypeTable;
use std::fmt::Debug;
use crate::compiler::ir::ast::{AstExpr, AstExprKind};

#[derive(Clone, Debug)]
pub struct ThirExpr<'a> {
    pub kind: ThirExprKind<'a>,
    pub ty: Type,
    pub span: Span<'a>,
}

impl<'a> ThirExpr<'a> {
    pub fn new(span: Span<'a>, ty: Type, kind: ThirExprKind<'a>) -> ThirExpr<'a> {
        ThirExpr { kind, ty, span }
    }
}

#[derive(Clone, Debug)]
pub enum ThirExprKind<'a> {
    /// Number
    Number(i32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<ThirExpr<'a>>),

    /// An identifier
    Identifier(SymbolId),

    /// A binary operation between two expressions
    BinaryOp { left: Box<ThirExpr<'a>>, operation: Operation, right: Box<ThirExpr<'a>> },

    /// Address of a variable
    AddressOf(SymbolId),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<ThirExpr<'a>>),

    /// Struct literal like MyStruct { a: 10 }
    StructLiteral {
        ty_id: StructId,
        values: Vec<ThirExpr<'a>>
    },

    /// Function call
    FunctionCall { function: Box<ThirExpr<'a>>, arguments: Vec<ThirExpr<'a>> },

    /// Index expression
    IndexAccess { left: Box<ThirExpr<'a>>, index: Box<ThirExpr<'a>> },

    /// Break
    Break { value: Option<Box<ThirExpr<'a>>> },

    /// Continue
    Continue { value: Option<Box<ThirExpr<'a>>> },

    /// An if statement
    IfExpression {
        condition: Box<ThirExpr<'a>>,
        body: Box<ThirBlock<'a>>,

        /// Optional else condition, else block
        else_statements: Vec< (Option<Box<ThirExpr<'a>>>, Box<ThirBlock<'a>>) >,
    },

    /// Unconditional loop
    Loop {
        body: Box<ThirBlock<'a>>,
    },

    /// A Block expression
    BlockExpression(Box<ThirBlock<'a>>),
}


impl<'a> ThirExpr<'a> {
    pub fn pretty_print(&self, type_table: &TypeTable) -> String {
        let expression_string = match &self.kind {
            ThirExprKind::Number(number) => number.to_string(),
            ThirExprKind::String(string) => format!("{:?}", string),
            ThirExprKind::Char(character) => format!("{:?}", character),
            ThirExprKind::Bool(boolean) => boolean.to_string(),

            ThirExprKind::ArrayLiteral(elements) => {
                let element_strings: Vec<String> = elements
                    .iter()
                    .map(|element| element.pretty_print(type_table))
                    .collect();
                format!("[{}]", element_strings.join(", "))
            }

            ThirExprKind::Identifier(symbol_id) => format!("id{}", symbol_id.0),

            ThirExprKind::BinaryOp { left, operation, right } => format!(
                "({} {} {})",
                left.pretty_print(type_table),
                operation,
                right.pretty_print(type_table)
            ),

            ThirExprKind::AddressOf(symbol_id) => format!("&{:?}", symbol_id),
            ThirExprKind::Dereference(inner_expression) => {
                format!("*{}", inner_expression.pretty_print(type_table))
            }
            ThirExprKind::StructLiteral { ty_id,  values } => {
                format!("struct_{} {{\n{:#?}\n}}", ty_id.0, values)
            },

            ThirExprKind::FunctionCall { function, arguments } => {
                let argument_strings: Vec<String> = arguments
                    .iter()
                    .map(|argument| argument.pretty_print(type_table))
                    .collect();
                format!(
                    "{}({})",
                    function.pretty_print(type_table),
                    argument_strings.join(", ")
                )
            }

            ThirExprKind::IndexAccess { left, index } => format!(
                "{}[{}]",
                left.pretty_print(type_table),
                index.pretty_print(type_table)
            ),

            ThirExprKind::Break { value } => match value {
                Some(break_value) => format!("break {}", break_value.pretty_print(type_table)),
                None => "break".to_string(),
            },
            ThirExprKind::Continue { value } => match value {
                Some(continue_value) => {
                    format!("continue {}", continue_value.pretty_print(type_table))
                }
                None => "continue".to_string(),
            },

            ThirExprKind::IfExpression { condition, body, else_statements } => {
                let mut result = format!(
                    "if {} {}",
                    condition.pretty_print(type_table),
                    body.pretty_print(type_table)
                );
                for (else_condition, else_body) in else_statements {
                    match else_condition {
                        Some(condition_expression) => result.push_str(&format!(
                            " else if {} {}",
                            condition_expression.pretty_print(type_table),
                            else_body.pretty_print(type_table)
                        )),
                        None => result.push_str(&format!(
                            " else {}",
                            else_body.pretty_print(type_table)
                        )),
                    }
                }
                result
            }

            ThirExprKind::Loop { body } => format!("loop {}", body.pretty_print(type_table)),
            ThirExprKind::BlockExpression(block) => block.pretty_print(type_table),
        };

        format!("{} typed: {}", expression_string, type_table.type_as_string(&self.ty.id))
    }
}