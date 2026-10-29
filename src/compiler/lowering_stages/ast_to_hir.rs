use std::collections::HashMap;
use crate::compiler::errors::*;
use crate::compiler::ir::ast::*;
use crate::compiler::ir::hir::*;


pub struct AstToHir {
    scopes: Vec<HashMap<String, SymbolId>>
}

impl<'a> AstToHir {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn lower(&mut self, ast: Vec<AstStmt<'a>>) -> Result<Vec<HirStmt<'a>>, CompilerError<'a>> {
        // First we collect type definitions in typedef_table and declare the function item names


        // Second we create the hir which can now fill out the type table and use function names
        let mut hir = Vec::new();

        for stmt in ast {
           hir.push(self.lower_stmt(stmt)?);
        }

        Ok(hir)
    }

    fn lower_stmt(&mut self, node: AstStmt<'a>) -> Result<HirStmt<'a>, CompilerError<'a>> {
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

                HirStmtKind::VariableDeclaration {
                    symbol_id,
                    value: lowered_value,
                    var_type: var_type.clone(),
                }
            },
            AstStmtKind::Item(item) => {
                HirStmtKind::Item(Box::new(self.lower_item(*item)?))
            },
            AstStmtKind::Expression(expr) => {
                HirStmtKind::Expression(Box::new(self.lower_expr(*expr)?))
            },
            AstStmtKind::Semi(expr) => {
                HirStmtKind::Semi(Box::new(self.lower_expr(*expr)?))
            }
        };

        Ok(HirStmt::new(node.span, kind))
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

                let loop_stmt = HirStmt::new(
                    node.span,
                    HirStmtKind::Expression(
                        Box::new(HirExpr::new(
                            node.span,
                            HirExprKind::Loop {
                                body: Box::new(HirBlock::new(
                                    vec![
                                        if_stmt,
                                        self.lower_stmt(*update)?
                                    ],
                                    node.span
                                ))
                            }
                        ))
                    )
                );

                HirExprKind::BlockExpression(
                    Box::new(HirBlock::new(
                        vec![
                            lowered_init,
                            loop_stmt
                        ], node.span
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

    fn lower_item(&mut self, node: AstItem<'a>) -> Result<HirItem<'a>, CompilerError<'a>> {
        let item = match node.kind {
            AstItemKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                for arg in &arguments {
                    self.declare_variable(&arg.name);
                }

                let lowered_body = Box::new(self.lower_block(body)?);

                HirItemKind::FunctionDeclaration {
                    name,
                    ret_type,
                    arguments,
                    body: lowered_body,
                }
            },
            AstItemKind::StructDefinition { name, fields } => {
                HirItemKind::StructDefinition { name, fields }
            },
            AstItemKind::TypeDeclaration { name, new_type } => {
                HirItemKind::TypeDeclaration { name, new_type }
            }
        };

        Ok(HirItem::new(node.span, item))
    }

    fn lower_block(&mut self, node: AstBlock<'a>) -> Result<HirBlock<'a>, CompilerError<'a>> {
        self.enter_scope();

        let mut lowered_stmts = Vec::with_capacity(node.statements.len());
        for stmt in node.statements {
            lowered_stmts.push(self.lower_stmt(stmt)?);
        }

        self.pop_scope();

        Ok(HirBlock::new(
            lowered_stmts,
            node.span
        ))
    }



    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: &str) -> SymbolId {
        let id = SymbolId(uid::Id::<u16>::new());
        self.scopes.last_mut().unwrap().insert(name.to_string(), id.clone());
        id
    }

    fn get_var_id(&self, name: &str) -> Option<SymbolId> {
        println!("{}", self.scopes.len());
        for i in (0..self.scopes.len()).into_iter().rev() {
            println!("{:?}", self.scopes[i]);
            let scope = &self.scopes[i];
            if !scope.contains_key(name) { continue; }
            return scope.get(name).cloned();
        }
        None
    }

    fn resolve_variable(&mut self, name: &str, span: Span<'a>) -> Result<SymbolId, CompilerError<'a>> {
        let symbol_id = self.get_var_id(&name);
        if symbol_id.is_none() {
            println!("{:?}", self.scopes);
            return Err(CompilerError::new(
                span,
                "Invalid identifier, not found."
            ))
        }
        Ok(symbol_id.unwrap())
    }
}