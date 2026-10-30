use std::collections::HashMap;
use std::env::var;
use crate::compiler::errors::*;
use crate::compiler::ir::ast::*;
use crate::compiler::ir::hir::*;
use crate::compiler::ir::types::*;

pub struct AstToHir<'a> {
    symbol_table: Vec<HashMap<String, SymbolId>>,
    typedef_table: Vec<HashMap<String, TypeDefinition<'a>>>,
    pub type_table: HashMap<SymbolId, ResolvedType>,
}

impl<'a> AstToHir<'a> {
    pub fn new() -> Self {
        Self {
            symbol_table: vec![HashMap::new()],
            typedef_table: vec![HashMap::new()],
            type_table: HashMap::new(),
        }
    }

    pub fn lower(&mut self, ast: Vec<AstStmt<'a>>) -> Result<Vec<HirStmt<'a>>, CompilerError<'a>> {
        // First we collect type definitions in typedef_table and declare the function item names
        for stmt in &ast {
            match &stmt.kind {
                AstStmtKind::Item(item) => {
                    self.register_typedef(*item.clone())?;
                },
                _ => {}
            }
        }

        println!("here {:?}", self.typedef_table);

        // Second we create the hir which can now fill out the type table and use function names
        // Also resolves sizeof(t) to t.get_size()
        let mut hir = Vec::new();

        for stmt in ast {
            let lowered = self.lower_stmt(stmt)?;
            if let Some(lowered) = lowered {
                hir.push(lowered);
            }
        }

        Ok(hir)
    }

    fn lower_stmt(&mut self, node: AstStmt<'a>) -> Result<Option<HirStmt<'a>>, CompilerError<'a>> {
        let kind: HirStmtKind = match node.kind {
            AstStmtKind::Assignment { target, value } => {
                HirStmtKind::Assignment {
                    target: Box::new(self.lower_expr(*target)?),
                    value: Box::new(self.lower_expr(*value)?),
                }
            },
            AstStmtKind::VariableDeclaration { name, value, var_type} => {
                let lowered_value = if value.is_none() { None } else { Some(Box::new(self.lower_expr(*value.unwrap())?)) };
                let symbol_id = self.declare_variable(&name);

                let resolved_type = self.resolve_type(&var_type, node.span, None)?;
                self.type_table.insert(symbol_id.clone(), resolved_type);

                HirStmtKind::VariableDeclaration {
                    symbol_id,
                    value: lowered_value,
                    var_type: var_type.clone(),
                }
            },
            AstStmtKind::Item(item) => {
                let item = self.lower_item(*item)?;
                if item.is_none() {
                    return Ok(None);
                } else {
                    HirStmtKind::Item(Box::new(item.unwrap()))
                }
            },
            AstStmtKind::Expression(expr) => {
                HirStmtKind::Expression(Box::new(self.lower_expr(*expr)?))
            },
            AstStmtKind::Semi(expr) => {
                HirStmtKind::Semi(Box::new(self.lower_expr(*expr)?))
            }
        };

        Ok(Some(HirStmt::new(node.span, kind)))
    }

    fn lower_expr(&mut self, node: AstExpr<'a>) -> Result<HirExpr<'a>, CompilerError<'a>> {
        let kind: HirExprKind = match node.kind {
            AstExprKind::Continue { value } => {
                HirExprKind::Continue { value: if value.is_some() { Some(Box::new(self.lower_expr(*value.unwrap())?)) } else { None } }
            },
            AstExprKind::Break { value } => {
                HirExprKind::Break { value: if value.is_some() { Some(Box::new(self.lower_expr(*value.unwrap())?)) } else { None } }
            },
            AstExprKind::Identifier(name) => {
                HirExprKind::Identifier(self.resolve_variable(&name, node.span)?)
            },
            AstExprKind::AddressOf(var_name) => {
                HirExprKind::AddressOf(self.resolve_variable(&var_name, node.span)?)
            },
            AstExprKind::Dereference(ast_node) => {
                HirExprKind::Dereference(Box::new(self.lower_expr(*ast_node)?))
            },
            AstExprKind::FunctionCall { function, arguments } => {
                let mut lowered_arguments = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    lowered_arguments.push(self.lower_expr(arg)?);
                }

                HirExprKind::FunctionCall {
                    function: Box::new(self.lower_expr(*function)?),
                    arguments: lowered_arguments,
                }
            },
            AstExprKind::ArrayLiteral(items) => {
                let mut lowered_items = Vec::with_capacity(items.len());
                for item in items {
                    lowered_items.push(self.lower_expr(item)?);
                }

                HirExprKind::ArrayLiteral(lowered_items)
            },
            AstExprKind::BinaryOp { left, operation, right} => {
                HirExprKind::BinaryOp {
                    left: Box::new(self.lower_expr(*left)?),
                    operation,
                    right: Box::new(self.lower_expr(*right)?),
                }
            },
            AstExprKind::BlockExpression(block) => {

                HirExprKind::BlockExpression(Box::new(self.lower_block(*block)?))
            },
            AstExprKind::IfExpression { condition, body, else_statements } => {
                let mut lowered_else_statements = Vec::with_capacity(else_statements.len());
                for else_statement in else_statements {
                    let lowered_condition = if else_statement.0.is_some() {Some(Box::new(self.lower_expr(*else_statement.0.unwrap())?)) }
                        else { None };

                    lowered_else_statements.push((lowered_condition, Box::new(self.lower_block(*else_statement.1)?)));
                }

                HirExprKind::IfExpression {
                    condition: Box::new(self.lower_expr(*condition)?),
                    body: Box::new(self.lower_block(*body)?),
                    else_statements: lowered_else_statements,
                }
            },
            AstExprKind::Loop { body } => {
                HirExprKind::Loop {
                    body: Box::new(self.lower_block(*body)?),
                }
            },
            AstExprKind::Bool(value) => HirExprKind::Bool(value),
            AstExprKind::Char(value) => HirExprKind::Char(value),
            AstExprKind::String(value) => HirExprKind::String(value),
            AstExprKind::Number(value) => HirExprKind::Number(value),
            AstExprKind::IndexAccess { left, index } => {
                HirExprKind::IndexAccess {
                    left: Box::new(self.lower_expr(*left)?),
                    index: Box::new(self.lower_expr(*index)?),
                }
            },
            AstExprKind::ForLoop { init, condition, update, body } => {
                #[allow(unused_doc_comments)]
                /// A for loop should be lowered to
                /// {
                ///     init
                ///     loop  {
                ///         if (condition) { body }
                ///         else { break; }
                ///         update;
                ///     }
                /// }

                let lowered_init = self.lower_stmt(*init)?;

                // block node with break
                let else_body = HirBlock::new(
                    vec![
                    HirStmt::new(
                        node.span,
                        HirStmtKind::Semi(
                            Box::new(HirExpr::new(
                                node.span,
                                HirExprKind::Break { value: None }
                            ))
                        )
                    )
                ], node.span);

                let if_stmt = HirStmt::new(
                    node.span,
                    HirStmtKind::Semi(
                        Box::new(HirExpr::new(
                            node.span,
                            HirExprKind::IfExpression {
                                condition: Box::new(self.lower_expr(*condition)?),
                                body: Box::new(self.lower_block(*body)?),
                                else_statements: vec![(None, Box::new(else_body))]
                            }
                        ))
                    )
                );

                let mut stmts = Vec::with_capacity(2);
                let lowered_update = self.lower_stmt(*update)?;
                if let Some(init) = lowered_update {
                    stmts.push(init);
                }
                stmts.push(if_stmt);


                let loop_stmt = HirStmt::new(
                    node.span,
                    HirStmtKind::Expression(
                        Box::new(HirExpr::new(
                            node.span,
                            HirExprKind::Loop {
                                body: Box::new(HirBlock::new(
                                    stmts,
                                    node.span
                                ))
                            }
                        ))
                    )
                );

                let mut stmts = Vec::with_capacity(2);
                if let Some(init) = lowered_init {
                    stmts.push(init);
                }
                stmts.push(loop_stmt);

                HirExprKind::BlockExpression(
                    Box::new(HirBlock::new(
                        stmts, node.span
                    ))
                )
            },
            AstExprKind::WhileLoop { condition, body } => {
                #[allow(unused_doc_comments)]
                /// While loops become
                /// loop {
                ///     if (condition) { body }
                ///     else { break; }
                /// }

                let else_body = HirBlock::new(
                    vec![
                        HirStmt::new(
                            node.span,
                            HirStmtKind::Semi(
                                Box::new(HirExpr::new(
                                    node.span,
                                    HirExprKind::Break { value: None }
                                ))
                            )
                        )
                    ], node.span);

                let if_stmt = HirStmt::new(
                    node.span,
                    HirStmtKind::Semi(
                        Box::new(HirExpr::new(
                            node.span,
                            HirExprKind::IfExpression {
                                condition: Box::new(self.lower_expr(*condition)?),
                                body: Box::new(self.lower_block(*body)?),
                                else_statements: vec![(None, Box::new(else_body))]
                            }
                        ))
                    )
                );


                HirExprKind::Loop {
                    body: Box::new(HirBlock::new(
                        vec![if_stmt],
                        node.span
                    ))
                }
            }
            AstExprKind::TypeCast { new_type, expr } => {
                    HirExprKind::TypeCast {
                        new_type,
                        expr: Box::new(self.lower_expr(*expr)?),
                    }
            }
        };

        Ok(HirExpr::new(node.span, kind))
    }

    fn lower_item(&mut self, node: AstItem<'a>) -> Result<Option<HirItem<'a>>, CompilerError<'a>> {
        let item = match node.kind {
            AstItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                for arg in &arguments {
                    self.declare_variable(&arg.name);
                }

                let lowered_body = Box::new(self.lower_block(body)?);

                Some(HirItemKind::FunctionDeclaration {
                    name,
                    ret_type,
                    arguments,
                    body: lowered_body,
                })
            },
            AstItemKind::StructDefinition { name, fields } => {
                None
            },
            AstItemKind::TypeDeclaration { name, new_type } => {
                None
            }
        };

        if let Some(item) = item {
            Ok(Some(HirItem::new(node.span, item)))
        } else {
            Ok(None)
        }
    }

    fn lower_block(&mut self, node: AstBlock<'a>) -> Result<HirBlock<'a>, CompilerError<'a>> {
        self.enter_scope();

        let mut lowered_stmts = Vec::with_capacity(node.statements.len());
        for stmt in node.statements {
            if let Some(stmt) = self.lower_stmt(stmt)? {
                lowered_stmts.push(stmt);
            }
        }

        self.pop_scope();

        Ok(HirBlock::new(
            lowered_stmts,
            node.span
        ))
    }



    fn enter_scope(&mut self) {
        self.symbol_table.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop();
    }

    fn declare_variable(&mut self, name: &str) -> SymbolId {
        let id = SymbolId(uid::Id::<u16>::new());
        self.symbol_table.last_mut().unwrap().insert(name.to_string(), id.clone());
        id
    }

    fn get_var_id(&self, name: &str) -> Option<SymbolId> {
        for i in (0..self.symbol_table.len()).into_iter().rev() {
            let scope = &self.symbol_table[i];
            if !scope.contains_key(name) { continue; }
            return scope.get(name).cloned();
        }
        None
    }

    fn resolve_variable(&mut self, name: &str, span: Span<'a>) -> Result<SymbolId, CompilerError<'a>> {
        let symbol_id = self.get_var_id(&name);
        if symbol_id.is_none() {
            return Err(CompilerError::new(
                span,
                "Invalid identifier, not found."
            ))
        }
        Ok(symbol_id.unwrap())
    }


    fn register_typedef(&mut self, item: AstItem<'a>) -> Result<(), CompilerError<'a>> {
        match &item.kind {
            AstItemKind::StructDefinition { name, fields } => {
                let typedef = TypeDefinition {
                    name: name.clone(),
                    kind: TypeKind::Struct {
                        fields: fields.clone(),
                    },
                };

                self.typedef_table.last_mut().unwrap().insert(name.clone(), typedef);

                for field in fields {
                    // Check for circular dependency
                    self.resolve_type(&field.field_type, field.span, Some(&name))?;
                }
            },
            AstItemKind::TypeDeclaration { name, new_type } => {
                let typedef = TypeDefinition {
                    name: name.clone(),
                    kind: TypeKind::Alias(new_type.clone()),
                };

                self.typedef_table.last_mut().unwrap().insert(name.clone(), typedef);
            },
            _ => {}
        }

        Ok(())
    }

    fn resolve_type(&self, unresolved: &UnresolvedType, span: Span<'a>, origin_primitive: Option<&str>) -> Result<ResolvedType, CompilerError<'a>> {
        match unresolved {
            UnresolvedType::Primitive(name) => {
                // Check for primitive
                if let Some(primitive) = ResolvedPrimitive::from_str(name) {
                    return Ok(ResolvedType::Primitive(primitive));
                };

                // Check for circular dependency
                if let Some(origin_primitive) = origin_primitive {
                    if *name == origin_primitive {
                        return Err(CompilerError::new(
                            span,
                            "Could not resolve type due to circular type dependency"
                        ))
                    }
                }


                let new_type = self.typedef_table.last().unwrap().get(name);
                if new_type.is_none() {
                    return Err(CompilerError::new(
                        span,
                        format!("Could not resolve type {}.", name).as_str(),
                    ));
                }
                let new_type = new_type.unwrap();

                let resolved_type = match &new_type.kind {
                    TypeKind::Alias(ty) => {
                        self.resolve_type(&ty, span, Some(origin_primitive.unwrap_or(&name)))?
                    },
                    TypeKind::Struct { fields } => {
                        let mut resolved_fields = Vec::new();
                        for field in fields {
                            let resolved_field_type = self.resolve_type(&field.field_type, span, Some(origin_primitive.unwrap_or(&name)))?;
                            resolved_fields.push(ResolvedField {
                                name: field.name.clone(),
                                field_type: resolved_field_type,
                            });
                        }
                        ResolvedType::Struct(resolved_fields)
                    },
                    _ => { return Err(CompilerError::new(
                        span,
                        "Not implemented yet."
                    ))}
                };

                Ok(resolved_type)
            },
            UnresolvedType::Array { length, inner_type } => {
                if length.is_none() {
                    return Err(CompilerError::new(
                        span,
                        "Could not resolve type, undeclared array length."
                    ));
                }

                let resolved_inner = self.resolve_type(inner_type, span, origin_primitive)?;

                Ok(ResolvedType::Array {
                    length: length.unwrap(),
                    inner_type: Box::new(resolved_inner)
                })
            },
            UnresolvedType::Pointer(inner_type) => {
                Ok(ResolvedType::Pointer(Box::new(*inner_type.clone())))
            },
            UnresolvedType::Function { ret_type, params } => {
                let resolved_ret_type = self.resolve_type(ret_type, span, origin_primitive)?;
                let mut resolved_args = Vec::new();

                for param in params {
                    resolved_args.push(self.resolve_type(param, span, origin_primitive)?);
                }

                Ok(ResolvedType::Function {
                    ret_type: Box::new(resolved_ret_type),
                    params: resolved_args,
                })
            }
        }
    }
}