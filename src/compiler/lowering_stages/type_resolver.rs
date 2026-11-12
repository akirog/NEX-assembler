use crate::compiler::errors::{CompilerError, Span};
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::TypeId;

pub struct TypeResolver<'type_table> {
    pub type_table: &'type_table mut TypeTable,
}

impl<'thir, 'type_table> TypeResolver<'type_table> {
    pub fn new(type_table: &'type_table mut TypeTable) -> Self {
        Self { type_table }
    }

    pub fn resolve(&mut self, thir: &mut [ThirStmt<'thir>]) -> Result<(), CompilerError<'thir>> {
        for statement in thir {
            self.resolve_stmt(statement)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &mut ThirStmt<'thir>) -> Result<(), CompilerError<'thir>> {
        match &mut statement.kind {
            ThirStmtKind::Item(item) => { self.resolve_item(item)?; },
            ThirStmtKind::VariableDeclaration { value, ty, symbol_id } => {
                if let Some(value) = value {
                    self.resolve_expr(value)?;
                }

                ty.id = self.try_resolve_type(ty.id, statement.span)?;

                self.type_table.register_symbol_type(symbol_id, &ty.id)

            }
            ThirStmtKind::Return { value, ty } => {
                if let Some(value) = value {
                    self.resolve_expr(value)?;
                }

                ty.id = self.try_resolve_type(ty.id, statement.span)?;
            }
            ThirStmtKind::Expression(expression) => {
                self.resolve_expr(expression)?;
            }
            ThirStmtKind::Semi(inner_statement) => {
                self.resolve_expr(inner_statement)?;
            }
            ThirStmtKind::Assignment { target, value, ty } => {
                self.resolve_expr(target)?;
                self.resolve_expr(value)?;

                ty.id = self.try_resolve_type(ty.id, statement.span)?;
            }
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expression: &mut ThirExpr<'thir>) -> Result<(), CompilerError<'thir>> {
        expression.ty.id = self.try_resolve_type(expression.ty.id, expression.span)?;

        match &mut expression.kind {
            ThirExprKind::Number(_)
            | ThirExprKind::String(_)
            | ThirExprKind::Char(_)
            | ThirExprKind::Bool(_)
            | ThirExprKind::Identifier(_)
            | ThirExprKind::AddressOf(_) => {}

            ThirExprKind::ArrayLiteral(items) => {
                for item in items {
                    self.resolve_expr(item)?;
                }
            }
            ThirExprKind::BinaryOp { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            ThirExprKind::Dereference(inner) => {
                self.resolve_expr(inner)?;
            }
            ThirExprKind::StructLiteral { ty, fields } => {
                *ty = self.try_resolve_type(*ty, expression.span)?;
                for field in fields {
                    self.resolve_expr(&mut field.value)?;
                }
            }
            ThirExprKind::MemberAccess { left, .. } => {
                self.resolve_expr(left)?;
            }
            ThirExprKind::FunctionCall { function, arguments } => {
                self.resolve_expr(function)?;
                for argument in arguments {
                    self.resolve_expr(argument)?;
                }
            }
            ThirExprKind::IndexAccess { left, index } => {
                self.resolve_expr(left)?;
                self.resolve_expr(index)?;
            }
            ThirExprKind::Break { value } | ThirExprKind::Continue { value } => {
                if let Some(value) = value {
                    self.resolve_expr(value)?;
                }
            }
            ThirExprKind::IfExpression { condition, body, else_statements } => {
                self.resolve_expr(condition)?;
                self.resolve_block(body)?;
                for (else_condition, else_body) in else_statements {
                    if let Some(else_condition) = else_condition {
                        self.resolve_expr(else_condition)?;
                    }
                    self.resolve_block(else_body)?;
                }
            }
            ThirExprKind::Loop { body } => {
                self.resolve_block(body)?;
            }
            ThirExprKind::BlockExpression(block) => {
                self.resolve_block(block)?;
            }
        }


        Ok(())
    }

    fn resolve_item(&mut self, item: &mut ThirItem<'thir>) -> Result<(), CompilerError<'thir>> {
        match &mut item.kind {
            ThirItemKind::FunctionDeclaration { body, .. } => {
                self.resolve_block(body)?;
            }
        }

        Ok(())
    }

    fn resolve_block(&mut self, block: &mut ThirBlock<'thir>) -> Result<(), CompilerError<'thir>> {
        for statement in &mut block.statements {
            self.resolve_stmt(statement)?;
        }

        Ok(())
    }

    fn try_resolve_type(&mut self, ty: TypeId, span: Span<'thir>) -> Result<TypeId, CompilerError<'thir>> {
        let Some(new_ty) = self.type_table.resolve_type(ty) else {
            return Err(CompilerError::new(
                span,
                "Could not resolve type of expression"
            ))
        };

        Ok(new_ty)
    }
}