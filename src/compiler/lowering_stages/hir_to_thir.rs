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
                let ty = if let Some(var_type) = var_type {
                    self.type_table.get_written_type_id(&var_type)?
                } else {
                    self.type_table.get_new_unresolved()
                };

                self.type_table.register_symbol_type(&symbol_id, &ty);

                let mut lowered_value = if let Some(value) = value {
                    Some(self.box_expr(*value)?)
                } else {
                    None
                };

                if let Some(value) = &mut lowered_value {
                    let result = self.try_coerce(value, ty);
                    if result.is_err() {
                        return Err(CompilerError::new(stmt.span,
                        "Declaration with mismatched types"))
                    }
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
                let target = self.lower_expr(*target)?;
                let value = self.lower_expr(*value)?;

                let result = self.type_table.unify(target.ty.id, value.ty.id);
                if result.is_err() {
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
                        id: self.type_table.get_new_unresolved_num()
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
            HirExprKind::StructLiteral { name, fields } => {
                let Some(type_id) = self.type_table.get_struct_type_id(&name) else {
                    return Err(CompilerError::new(
                        expr.span,
                        "Invalid struct name"
                    ))
                };

                let type_kind = self.type_table.get_type(&type_id);
                let names = match type_kind {
                    TypeKind::Struct { fields } => { fields.names().to_vec() },
                    _ => return Err(CompilerError::new(
                        expr.span,
                        "Invalid struct type"
                    ))
                };

                let mut lowered_fields = Vec::with_capacity(fields.len());

                let mut count_found = 0;
                for field in fields {
                    let mut found = false;
                    for (i, field_name) in names.iter().enumerate() {
                        if field.name == *field_name {
                            if found {
                                return Err(CompilerError::new(
                                    field.span,
                                    "Duplicate field name"
                                ))
                            }

                            let expr = ThirFieldExpr {
                                name: field.name.clone(),
                                value: Box::from(self.lower_expr(*field.value.clone())?),
                                span: field.span
                            };

                            if i < lowered_fields.len() {
                                lowered_fields.insert(i, expr);
                            } else {
                                lowered_fields.push(expr);
                            }

                            found = true;
                            count_found += 1;
                        }
                    }
                    if !found {
                        return Err(CompilerError::new(
                            field.span,
                            "Invalid field name"
                        ))
                    }
                }

                if count_found < names.len() {
                    return Err(CompilerError::new(
                        expr.span,
                        "Missing fields in struct literal"
                    ))
                }


                (
                    Type { id: type_id.clone() },
                    ThirExprKind::StructLiteral {
                        ty: type_id,
                        fields: lowered_fields,
                    }
                )
            },
            HirExprKind::MemberAccess { left, member } => {
                let left = self.lower_expr(*left)?;
                let Some(member_type) = self.type_table.get_member_type(&left.ty.id, &member) else {
                    return Err(CompilerError::new(
                        expr.span,
                        "Invalid member access"
                    ));
                };

                (
                    Type { id: member_type },
                    ThirExprKind::MemberAccess {
                        left: Box::new(left),
                        member,
                    }
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
                let mut lowered_left = self.box_expr(*left)?;

                lowered_left.ty.id = self.type_table.resolve_type(lowered_left.ty.id).unwrap();
                let indexed_type = self.type_table.dereference_type(&lowered_left.ty.id);
                if indexed_type.is_none() {
                    return Err(CompilerError::new(
                        lowered_left.span,
                        "Cannot index this type"
                    ))
                }
                let indexed_type = indexed_type.unwrap();

                (
                    Type {
                        id: indexed_type
                    },
                    ThirExprKind::IndexAccess { left: lowered_left, index: self.box_expr(*index)? }
                )
            },
            HirExprKind::BinaryOp { left, operation, right } => {
                let left = self.lower_expr(*left)?;
                let right = self.lower_expr(*right)?;

                let ty = self.type_table.unify(left.ty.id, right.ty.id);
                if ty.is_err() {
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

                let (ret_ty, arg_types) = match self.type_table.get_type(&function.ty.id) {
                    TypeKind::Function { ret_ty, params } => (ret_ty.clone(), params),
                    TypeKind::FuncPtr { ret_ty, params } => (ret_ty.clone(), params),
                    _ => { return Err(CompilerError::new(
                        function.span,
                        "Cannot call var of this type"
                    ))}
                };

                if arg_types.len() != arguments.len() {
                    return Err(CompilerError::new(
                        function.span,
                        "Wrong param count in function call"
                    ))
                }

                let mut i = 0;
                for arg in arguments {
                    let arg_span = arg.span.clone();
                    let mut lowered_arg = self.lower_expr(arg)?;

                    let result = self.try_coerce(&mut lowered_arg, arg_types[i]);
                    if result.is_err() {
                        return Err(CompilerError::new(
                            arg_span,
                            "Could not coerce types"
                        ))
                    }

                    lowered_arguments.push(lowered_arg);
                    i += 1;
                }


                (
                    Type {
                        id: ret_ty
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
                let mut param_types = Vec::with_capacity(arguments.len());
                let mut param_ids = Vec::with_capacity(arguments.len());

                for arg in &arguments {
                    let arg_ty = self.type_table.get_written_type_id(&arg.ty)?;
                    self.type_table.register_symbol_type(&arg.id, &arg_ty);
                    param_types.push(arg_ty);
                    param_ids.push(arg.id.clone());
                }

                let ret_ty = self.type_table.get_written_type_id(&ret_type)?;
                if self.type_table.is_aggregate(&ret_ty) {
                    return Err(CompilerError::new(
                        item.span,
                        "Cannot return an aggregate type yet, pass a pointer to return location instead."
                    ))
                }

                let type_kind = TypeKind::Function {
                    ret_ty,
                    params: param_types,
                };

                let type_id = self.type_table.get_type_kind_id(&type_kind);
                self.type_table.register_symbol_type(&id, &type_id);

                ThirItemKind::FunctionDeclaration {
                    id, param_ids, ty: type_id, body: Box::new(self.lower_block(*body)?)
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

    fn try_coerce(&mut self, expr: &mut ThirExpr<'a>, target_type: TypeId) -> Result<(), ()> {
        // If they are the same type, aka can be unified, just return that
        let unified = self.type_table.unify(expr.ty.id, target_type);
        if unified.is_ok() {
            expr.ty.id = unified?;
            return Ok(());
        };


        let resolved = self.type_table.resolve_type(expr.ty.id);
        if resolved.is_none() {
            return Err(())
        };
        expr.ty.id = resolved.unwrap();
        let expr_ty = self.type_table.get_type(&expr.ty.id);


        let target_ty = self.type_table.get_type(&target_type);

        match (expr_ty, target_ty) {
            (TypeKind::Pointer(ptr_id), TypeKind::Slice(slice_id)) => {
                println!("ptr and slice");
                let arr_ty = match self.type_table.get_type(&ptr_id) {
                    TypeKind::Array { ty: arr_ty, ..} => {
                        arr_ty
                    },
                    _ => {
                        println!("wrong inner ptr type: {}", self.type_table.type_as_string(&ptr_id));
                        return Err(())
                    }
                };

                if arr_ty == slice_id {
                    *expr = ThirExpr::new(
                        expr.span,
                        Type { id: target_type },
                        ThirExprKind::ImplicitCoercion {
                            expr: Box::new(expr.clone()),
                            new_type: target_type
                        }
                    );
                    return Ok(());
                };

                Err(())
            },
            _ => Err(())
        }
    }
}