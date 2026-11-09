use crate::compiler::errors::CompilerError;
use crate::compiler::ir::hir::*;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::*;

pub struct HirLowerer {
    pub type_table: TypeTable,
}


impl<'a> HirLowerer {
    pub fn new(type_table: TypeTable) -> Self {
        Self {
            type_table,
        }
    }

    pub fn lower(&mut self, hir: Vec<HirStmt<'a>>) -> Result<Vec<ThirStmt<'a>>, CompilerError<'a>> {
        let mut thir = Vec::new();

        // First resolve function types
        for stmt in &hir {
            match &stmt.kind {
                HirStmtKind::Item(item) => {
                    match &item.kind {
                        HirItemKind::FunctionDeclaration { id, ret_type, arguments, ..} => {
                            let mut arg_ids = Vec::with_capacity(arguments.len());
                            for arg in arguments {
                                let arg_id = self.type_table.get_written_type_id(&arg.ty);
                                arg_ids.push(arg_id?);
                            }

                            let ret_ty = self.type_table.get_written_type_id(ret_type)?;

                            let ty = self.type_table.get_type_kind_id( &TypeKind::Function {
                                ret_ty,
                                params: arg_ids,
                            });

                            self.type_table.register_symbol_type(id, &ty);
                        },
                    }
                },
                _ => {}
            }
        }

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
                let mut ty = if let Some(var_type) = var_type {
                    self.type_table.get_written_type_id(&var_type)?
                } else {
                    self.type_table.get_type_kind_id(&TypeKind::Unresolved)
                };

                self.type_table.register_symbol_type(&symbol_id, &ty);

                let mut lowered_value = if let Some(value) = value {
                    Some(self.box_expr(*value)?)
                } else {
                    None
                };

                if lowered_value.is_none() && matches!(self.type_table.get_type(&ty), TypeKind::Unresolved) {
                    return Err(CompilerError::new(
                        stmt.span,
                        "Oh no"
                    ))
                } else if let Some(value) = &mut lowered_value && matches!(self.type_table.get_type(&ty), TypeKind::Unresolved) {
                    let result = self.type_table.enforce_same_type(&mut ty, &mut value.ty.id);
                    if result.is_none() {
                        return Err(CompilerError::new(stmt.span,
                        &"Declaration with mismatched types".to_string()))
                    }
                    self.type_table.register_symbol_type(&symbol_id, &ty);
                }

                ThirStmtKind::VariableDeclaration { symbol_id, value: lowered_value, ty: Type { id: ty } }
            },
            HirStmtKind::Return { value } => {
                if let Some(value) = value {
                    let value = self.lower_expr(*value)?;
                    let ty = value.ty.clone();

                    ThirStmtKind::Return { value: Some(Box::new(value)), ty }
                } else {
                    ThirStmtKind::Return { value: None, ty: Type { id: self.type_table.get_type_kind_id(&TypeKind::None) } }
                }
            },
            HirStmtKind::Expression(expr) => {
                ThirStmtKind::Expression(self.box_expr(*expr)?)
            },
            HirStmtKind::Semi(stmt) => {
                ThirStmtKind::Semi(self.box_expr(*stmt)?)
            },
            HirStmtKind::Assignment { target, value } => {
                let mut target = self.lower_expr(*target)?;
                let mut value = self.lower_expr(*value)?;

                let result = self.type_table.enforce_same_type(&mut target.ty.id, &mut value.ty.id);
                if result.is_none() {
                    return Err(CompilerError::new(
                        stmt.span,
                        &format!("Mismatched types in assignment\n\
                        Attempt to assign var with type {:?} to value with type {:?}", self.type_table.get_type(&target.ty.id), self.type_table.get_type(&value.ty.id))
                    ))
                }

                ThirStmtKind::Assignment { target: Box::new(target), value: Box::new(value), ty: Type { id: result.unwrap() } }
            },
        };

        Ok(ThirStmt::new(stmt.span, kind))
    }

    fn box_expr(&mut self, expr: HirExpr<'a>) -> Result<Box<ThirExpr<'a>>, CompilerError<'a>> {
        Ok(Box::new(self.lower_expr(expr)?))
    }

    fn lower_expr(&mut self, expr: HirExpr<'a>) -> Result<ThirExpr<'a>, CompilerError<'a>> {
        let (ty, kind) = match expr.kind {
            HirExprKind::String(string) => {
                let char_type = self.type_table.get_type_kind_id(&TypeKind::Char);

                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::Array {
                            ty: char_type,
                            len: string.len(),
                        })

                    },
                    ThirExprKind::String(string)
                )
            },
            HirExprKind::Char(char) => {
                (
                    Type {
                        id: self.type_table.get_type_kind_id( &TypeKind::Char)
                    },
                    ThirExprKind::Char(char)
                )
            },
            HirExprKind::Bool(value) => {
                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::Bool)
                    },
                    ThirExprKind::Bool(value)
                )
            },
            HirExprKind::Number(value) => {
                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::UnresolvedNum)
                    },
                    ThirExprKind::Number(value)
                )
            },
            HirExprKind::Identifier(id) => {
                let ty = self.type_table.get_symbol_type_id(&id);
                if ty.is_none() {
                    return Err(CompilerError::new(
                        expr.span,
                        "Type of var not found."
                    ))
                }
                let ty = ty.unwrap();


                (
                    Type {
                        id: ty,
                    },
                    ThirExprKind::Identifier(id)
                )
            },
            HirExprKind::ArrayLiteral(items) => {
                let mut lowered_items = Vec::with_capacity(items.len());
                let items_len = items.len();

                let mut ty = self.type_table.get_type_kind_id(&TypeKind::None);
                if items_len > 0 {
                    ty = self.lower_expr(items[0].clone())?.ty.id;

                    for item in items {
                        let item_span = item.span;
                        lowered_items.push(self.lower_expr(item)?);
                        if lowered_items.last().unwrap().ty.id != ty {
                            return Err(CompilerError::new(
                                item_span,
                                "Mismatched array type"
                            ))
                        }
                    }
                }

                (
                    Type {
                        id: self.type_table.get_type_kind_id( &TypeKind::Array { ty, len: items_len }),
                    },
                    ThirExprKind::ArrayLiteral(lowered_items)
                )
            },
            HirExprKind::IndexAccess { left, index } => {
                let lowered_left = self.box_expr(*left)?;

                if !self.type_table.can_index_type(&lowered_left.ty.id) {
                    return Err(CompilerError::new(
                        expr.span,
                        format!("Attempted to index a non indexable type. {:?}\n{:#?}", lowered_left.ty.id, self.type_table).as_str()
                    ))
                };

                let indexee_type = self.type_table.dereference_type(&lowered_left.ty.id);
                if indexee_type.is_none() {
                    return Err(CompilerError::new(
                        lowered_left.span,
                        "Cannot index this type"
                    ))
                }
                let indexee_type = indexee_type.unwrap();

                (
                    Type {
                        id: indexee_type
                    },
                    ThirExprKind::IndexAccess { left: lowered_left, index: self.box_expr(*index)? }
                )
            },
            HirExprKind::BinaryOp { left, operation, right } => {
                let mut left = self.lower_expr(*left)?;
                let mut right = self.lower_expr(*right)?;

                let ty = self.type_table.enforce_same_type(&mut left.ty.id, &mut right.ty.id);
                if ty.is_none() {
                    return Err(CompilerError::new(
                        left.span.extend(right.span),
                        &format!("Mismatched types in binary expression\nright: {:?}, left: {:?}\n{:#?}", left.ty.id, right.ty.id, self.type_table)
                    ))
                };
                let ty = ty.unwrap();

                (
                    Type {
                        id: ty
                    },
                    ThirExprKind::BinaryOp { left: Box::new(left), operation, right: Box::new(right) }
                )
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

                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::None)
                    },
                    ThirExprKind::IfExpression {
                        condition: lowered_condition,
                        body: lowered_body,
                        else_statements: lowered_else_statements
                    }
                )
            },
            HirExprKind::Continue { value } => {
                if let Some(value) = value {
                    (
                        Type {
                            id: self.type_table.get_type_kind_id(&TypeKind::None)
                        },
                        ThirExprKind::Continue { value: Some(self.box_expr(*value)?) }
                    )
                } else {
                    (
                        Type {
                            id: self.type_table.get_type_kind_id(&TypeKind::None)
                        },
                        ThirExprKind::Continue { value: None }
                    )
                }
            },
            HirExprKind::Break { value } => {
                if let Some(value) = value {
                    (
                        Type {
                            id: self.type_table.get_type_kind_id(&TypeKind::None)
                        },
                        ThirExprKind::Break { value: Some(self.box_expr(*value)?) }
                    )
                } else {
                    (
                        Type {
                            id: self.type_table.get_type_kind_id(&TypeKind::None)
                        },
                        ThirExprKind::Break { value: None }
                    )
                }
            },
            HirExprKind::Loop { body } => {
                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::None),
                    },
                    ThirExprKind::Loop { body: Box::new(self.lower_block(*body)?) }
                )
            },
            HirExprKind::Dereference(expr) => {
                let expr = self.lower_expr(*expr)?;
                let ty = self.type_table.dereference_type(&expr.ty.id);
                if ty.is_none() {
                    return Err(CompilerError::new(
                        expr.span,
                        "Invalid dereference."
                    ))
                };
                let ty = ty.unwrap();

                (
                    Type {
                        id: ty,
                    },
                    ThirExprKind::Dereference(Box::new(expr))
                )
            },
            HirExprKind::AddressOf(id) => {
                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::Pointer(
                            self.type_table.get_symbol_type_id(&id).unwrap()
                        ))
                    },
                    ThirExprKind::AddressOf(id)
                )
            },
            HirExprKind::FunctionCall { function, arguments } => {
                let function = self.lower_expr(*function)?;
                let mut lowered_arguments = Vec::with_capacity(arguments.len());

                for arg in arguments {
                    let lowered_arg = self.lower_expr(arg)?;
                    lowered_arguments.push(lowered_arg);
                }

                let ret_type = match self.type_table.get_type(&function.ty.id) {
                    TypeKind::FuncPtr { ret_ty, .. } => {
                        ret_ty
                    },
                    TypeKind::Function { ret_ty, .. } => {
                        ret_ty
                    },
                    _ => {
                        return Err(CompilerError::new(
                            function.span,
                            "Cannot call expression of this type"
                        ))
                    }
                };

                (
                    Type {
                        id: ret_type
                    },
                    ThirExprKind::FunctionCall { function: Box::new(function), arguments: lowered_arguments }
                )
            },
            HirExprKind::TypeCast { new_type, expr } => {
                (
                    Type {
                        id: self.type_table.get_written_type_id(&new_type)?
                    },
                    self.box_expr(*expr)?.kind
                )
            },
            HirExprKind::BlockExpression(block) => {
                (
                    Type {
                        id: self.type_table.get_type_kind_id(&TypeKind::None)
                    },
                    ThirExprKind::BlockExpression(Box::new(self.lower_block(*block)?))
                )
            }
        };

        Ok(ThirExpr::new(expr.span, ty, kind ))
    }

    fn lower_item(&mut self, item: HirItem<'a>) -> Result<ThirItem<'a>, CompilerError<'a>> {
        let kind = match item.kind {
            HirItemKind::FunctionDeclaration { id, ret_type, arguments, body } => {
                for arg in &arguments {
                    let arg_ty = self.type_table.get_written_type_id(&arg.ty)?;

                    self.type_table.register_symbol_type(&arg.id, &arg_ty);
                }

                let ret_type_id = self.type_table.get_written_type_id(&ret_type)?;
                self.type_table.register_symbol_type(&id, &ret_type_id);

                ThirItemKind::FunctionDeclaration {
                    id, arguments, body: Box::new(self.lower_block(*body)?)
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
}