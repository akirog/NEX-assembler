use crate::compiler::errors::Span;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::operations::*;
use crate::compiler::ir::thir::ThirBlock;
use crate::compiler::ir::types::resolved_type::*;
use crate::compiler::ir::types::struct_definition::StructId;
use crate::compiler::ir::types::type_table::TypeTable;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ThirExpr {
    pub kind: ThirExprKind,
    pub ty: Type,
    pub span: Span,
}

impl ThirExpr {
    pub fn new(span: Span, ty: Type, kind: ThirExprKind) -> ThirExpr {
        ThirExpr { kind, ty, span }
    }
}

#[derive(Clone, Debug)]
pub enum ThirExprKind {
    /// Number
    Number(i32),

    /// String
    String(String),

    /// Character
    Char(char),

    /// Boolean
    Bool(bool),

    /// Array literal
    ArrayLiteral(Vec<ThirExpr>),

    /// An identifier
    Identifier(SymbolId),

    /// A binary operation between two expressions
    BinaryOp { left: Box<ThirExpr>, operation: Operation, right: Box<ThirExpr> },

    /// A unary operation
    UnaryOp { operation: UnaryOperation, right: Box<ThirExpr> },

    /// Address of a variable
    AddressOf(Box<ThirExpr>),

    /// Memory dereference of a value, e.g. `*ptr`
    Dereference(Box<ThirExpr>),

    /// Struct literal like MyStruct { a: 10 }
    StructLiteral {
        ty: StructId,
        fields: Vec<ThirFieldExpr>
    },

    /// Member access like strooct.thing
    MemberAccess {
        left: Box<ThirExpr>,
        member: String,
    },

    /// An implicit coercion, like passing &arr to a function that wants a slice
    ImplicitCoercion {
        expr: Box<ThirExpr>,
        new_type: TypeId,
    },

    /// Function call
    FunctionCall { function: Box<ThirExpr>, arguments: Vec<ThirExpr> },

    /// Index expression
    IndexAccess { left: Box<ThirExpr>, index: Box<ThirExpr> },

    /// Break
    Break { value: Option<Box<ThirExpr>> },

    /// Continue
    Continue { value: Option<Box<ThirExpr>> },

    /// An if statement
    IfExpression {
        condition: Box<ThirExpr>,
        body: Box<ThirBlock>,

        /// Optional else condition, else block
        else_statements: Vec< (Option<Box<ThirExpr>>, Box<ThirBlock>) >,
    },

    /// Unconditional loop
    Loop {
        body: Box<ThirBlock>,
    },

    /// A Block expression
    BlockExpression(Box<ThirBlock>),
}


impl ThirExpr {
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
            ThirExprKind::UnaryOp { operation, right } => {
                format!("({}{})", operation, right.pretty_print(type_table))
            },

            ThirExprKind::AddressOf(symbol_id) => format!("&({})", symbol_id.pretty_print(type_table)),
            ThirExprKind::Dereference(inner_expression) => {
                format!("*{}", inner_expression.pretty_print(type_table))
            }
            ThirExprKind::StructLiteral { ty, fields: values } => {
                format!("struct_{:?} {{\n{}\n}}", ty, values.iter().map(|e| { e.pretty_print(type_table)}).collect::<Vec<_>>().join("\n") )
            },
            ThirExprKind::MemberAccess { left, member } => {
                format!("({}.{})", left.pretty_print(type_table), member)
            },

            ThirExprKind::ImplicitCoercion { expr, new_type } => {
                format!("({} coerced to {})", expr.pretty_print(type_table), type_table.type_as_string(new_type))
            }

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


#[derive(Clone, Debug)]
pub struct ThirFieldExpr {
    pub name: String,
    pub value: Box<ThirExpr>,
    pub span: Span,
}

impl ThirFieldExpr {
    pub(crate) fn pretty_print(&self, type_table: &TypeTable) -> String {
        format!("{}: {}", self.name, self.value.pretty_print(type_table))
    }
}