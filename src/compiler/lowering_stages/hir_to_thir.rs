use std::cmp::PartialEq;
use std::collections::HashMap;
use clap::arg;
use crate::compiler::errors::{CompilerError, Span};
use crate::compiler::ir::hir::*;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::{ResolvedPrimitive, ResolvedType};

pub struct HirLowerer {
    type_table: HashMap<SymbolId, ResolvedType>,
}


impl<'a> HirLowerer {
    pub fn new(type_table: HashMap<SymbolId, ResolvedType>) -> Self {
        Self {
            type_table
        }
    }

    pub fn lower(&mut self, hir: Vec<HirStmt<'a>>) -> Result<Vec<ThirStmt<'a>>, CompilerError<'a>> {
        let mut thir = Vec::new();

        for stmt in hir {
            thir.push(self.lower_stmt(stmt)?);
        }

        Ok(thir)
    }

    fn lower_stmt(&mut self, stmt: HirStmt<'a>) -> Result<ThirStmt<'a>, CompilerError<'a>> {
        let kind = match stmt.kind {
            HirStmtKind::Item(item) => {
                ThirStmtKind::Item(Box::new(self.lower_item(*item)?))
            },
            HirStmtKind::VariableDeclaration { symbol_id, value, var_type } => {
                let lowered_value = if let Some(value) = value {
                    Some(self.box_expr(*value)?)
                } else {
                    None
                };
                ThirStmtKind::VariableDeclaration { symbol_id, value: lowered_value, var_type }
            },
            HirStmtKind::Expression(expr) => {
                ThirStmtKind::Expression(self.box_expr(*expr)?)
            },
            HirStmtKind::Semi(stmt) => {
                ThirStmtKind::Semi(self.box_expr(*stmt)?)
            },
            HirStmtKind::Assignment { target, value } => {
                ThirStmtKind::Assignment { target: self.box_expr(*target)?, value: self.box_expr(*value)? }
            },
        };

        Ok(ThirStmt::new(stmt.span, kind))
    }

    fn box_expr(&mut self, expr: HirExpr<'a>) -> Result<Box<ThirExpr<'a>>, CompilerError<'a>> {
        Ok(Box::new(self.lower_expr(&expr)?))
    }

    fn lower_expr(&mut self, expr: &HirExpr<'a>) -> Result<ThirExpr<'a>, CompilerError<'a>> {
        let (ty, kind) = match expr.kind {
            HirExprKind::String(string) => {
                (
                    ResolvedType::Array {
                        length: string.len(),
                        inner_type: Box::new(ResolvedType::Primitive(ResolvedPrimitive::Char))
                    },
                    ThirExprKind::String(string)
                )
            },
            HirExprKind::Char(char) => {
                (
                    ResolvedType::Primitive(ResolvedPrimitive::Char),
                    ThirExprKind::Char(char)
                )
            },
            HirExprKind::Bool(value) => {
                (
                    ResolvedType::Primitive(ResolvedPrimitive::Bool),
                    ThirExprKind::Bool(value)
                )
            },
            HirExprKind::Number(value) => {
                (
                    ResolvedType::Primitive(ResolvedPrimitive::I32),
                    ThirExprKind::Number(value)
                )
            },
            HirExprKind::Identifier(id) => {
                (
                    *self.get_var_type(&id, expr.span)?,
                    ThirExprKind::Identifier(id)
                )
            },
            HirExprKind::ArrayLiteral(items) => {
                let mut lowered_items = Vec::with_capacity(items.len());
                
                let mut ty = ResolvedType::Primitive(ResolvedPrimitive::U8);
                if items.len() > 0 {
                    ty = self.lower_expr(&items[0])?.ty;
                    
                    for item in items {
                        lowered_items.push(self.lower_expr(&item)?);
                        if lowered_items.last().unwrap().ty != ty {
                            return Err(CompilerError::new(
                                item.span,
                                "Mismatched array type"
                            ))
                        }
                    }
                }
                
                (
                    ty,
                    ThirExprKind::ArrayLiteral(lowered_items)
                )
            },
            HirExprKind::IndexAccess { left, index } => {
                let lowered_left = self.box_expr(*left)?;
                
                if !lowered_left.ty.indexable() {
                    return Err(CompilerError::new(
                        expr.span,
                        "Attempted to index a non indexable type."
                    ))
                }
                
                (
                    lowered_left.ty,
                    ThirExprKind::IndexAccess { left: lowered_left, index: self.box_expr(*index)? }
                )
            },
            HirExprKind::BinaryOp { left, operation, right } => {
                ThirExprKind::BinaryOp { left: self.box_expr(*left)?, operation, right: self.box_expr(*right)? }
            },
            HirExprKind::IfExpression { condition, body, else_statements } => {
                let lowered_condition = self.box_expr(*condition)?;
                let lowered_body = Box::new(self.lower_block(*body)?);

                let mut lowered_else_statements = Vec::with_capacity(else_statements.len());
                for els in else_statements {
                    let cond = if let Some(cond) = els.0 {
                        Some(self.box_expr(*cond)?)
                    } else {
                        None
                    };

                    let else_body = Box::new(self.lower_block(*els.1)?);

                    lowered_else_statements.push((cond, else_body));
                }

                ThirExprKind::IfExpression {
                    condition: lowered_condition,
                    body: lowered_body,
                    else_statements: lowered_else_statements
                }
            },
            HirExprKind::Continue { value } => {
                if let Some(value) = value {
                    ThirExprKind::Continue { value: Some(self.box_expr(*value)?) }
                } else {
                    ThirExprKind::Continue { value: None }
                }
            },
            HirExprKind::Break { value } => {
                if let Some(value) = value {
                    ThirExprKind::Break { value: Some(self.box_expr(*value)?) }
                } else {
                    ThirExprKind::Break { value: None }
                }
            },
            HirExprKind::Loop { body } => {
                ThirExprKind::Loop { body: Box::new(self.lower_block(*body)?) }
            },
            HirExprKind::Dereference(expr) => {
                ThirExprKind::Dereference(self.box_expr(*expr)?)
            },
            HirExprKind::AddressOf(id) => {
                ThirExprKind::AddressOf(id)
            },
            HirExprKind::FunctionCall { function, arguments } => {
                let mut lowered_arguments = Vec::with_capacity(arguments.len());

                for arg in arguments {
                    let lowered_arg = self.lower_expr(arg)?;
                    lowered_arguments.push(lowered_arg);
                }

                ThirExprKind::FunctionCall { function: self.box_expr(*function)?, arguments: lowered_arguments }
            },
            HirExprKind::TypeCast { new_type, expr } => {
                ThirExprKind::TypeCast { new_type, expr: self.box_expr(*expr)? }
            },
            HirExprKind::BlockExpression(block) => {
                ThirExprKind::BlockExpression(Box::new(self.lower_block(*block)?))
            }
        };

        Ok(ThirExpr::new(expr.span, ty, kind))
    }

    fn lower_item(&mut self, item: HirItem<'a>) -> Result<ThirItem<'a>, CompilerError<'a>> {
        let kind = match item.kind {
            HirItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                ThirItemKind::FunctionDeclaration {
                    name, ret_type, arguments, body: Box::new(self.lower_block(*body)?)
                }
            }
        };

        Ok(ThirItem::new(item.span, kind))
    }

    fn lower_block(&mut self, block: HirBlock<'a>) -> Result<ThirBlock<'a>, CompilerError<'a>> {
        let mut statements = Vec::new();

        for statement in block.statements {
            statements.push(self.lower_stmt(statement)?);
        }

        Ok(ThirBlock::new(statements, block.span))
    }


    fn get_var_type(&self, id: &SymbolId, span: Span<'a>) -> Result<&ResolvedType, CompilerError<'a>> {
        let ty = self.type_table.get(id);
        if let Some(ty) = ty {
            Ok(ty)
        } else {
            Err(CompilerError::new(
                span,
                "Could not get var type."
            ))
        }
    }
}