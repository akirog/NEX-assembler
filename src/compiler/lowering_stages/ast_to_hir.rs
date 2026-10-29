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
        let mut hir = Vec::new();

        for stmt in ast {
           hir.push(self.visit_stmt(stmt)?);
        }

        Ok(hir)
    }

    fn visit_stmt(&mut self, node: AstStmt<'a>) -> Result<HirStmt<'a>, CompilerError<'a>> {
        match node.kind {
            AstStmtKind::Assignment { target, value } => {
                Ok(HirStmt::new(
                    node.span,
                    HirStmtKind::Assignment {
                        target: Box::new(self.visit_expr(*target)?),
                        value: Box::new(self.visit_expr(*value)?),
                    }
                ))
            },
            AstStmtKind::VariableDeclaration { name, value, var_type} => {
                let symbol_id = self.declare_variable(&name);

                Ok(HirStmt::new(
                    node.span,
                    HirStmtKind::VariableDeclaration {
                        symbol_id,
                        value: if value.is_none() { None } else { Some(Box::new(self.visit_expr(*value.unwrap())?)) },
                        var_type: var_type.clone(),
                    }
                ))
            },
            AstStmtKind::Item(item) => {
                Ok(HirStmt::new(
                    node.span,
                    HirStmtKind::Item(Box::new(self.visit_item(*item)?))
                ))
            },
            AstStmtKind::Expression(expr) => {
                Ok(HirStmt::new(
                    node.span,
                    HirStmtKind::Expression(Box::new(self.visit_expr(*expr)?))
                ))
            },
            AstStmtKind::Semi(expr) => {
                Ok(HirStmt::new(
                    node.span,
                    HirStmtKind::Semi(Box::new(self.visit_expr(*expr)?))
                ))
            }
        }
    }

    fn visit_expr(&mut self, node: AstExpr<'a>) -> Result<HirExpr<'a>, CompilerError<'a>> {
        match node.kind {
            AstExprKind::Continue { value } => {
                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::Continue { value: None }
                ))
            },
            AstExprKind::Break { value } => {
                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::Break { value: None }
                ))
            },
            AstExprKind::Identifier(name) => {
                let symbol_id = self.get_var_id(&name);
                if symbol_id.is_none() {
                    return Err(CompilerError::new(
                        node.span,
                        "Invalid identifier, not found."
                    ))
                }

                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::Identifier(symbol_id.unwrap())
                ))
            },
            AstExprKind::AddressOf(var_name) => {
                let symbol_id = self.get_var_id(&var_name);
                if symbol_id.is_none() {
                    return Err(CompilerError::new(
                        node.span,
                        "Invalid identifier, not found."
                    ))
                }

                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::AddressOf(symbol_id.unwrap())
                ))
            },
            AstExprKind::Dereference(node) => {
                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::Dereference(Box::new(self.visit_expr(*node)?))
                ))
            },
            AstExprKind::FunctionCall { function, arguments } => {
                let mut lowered_arguments = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    lowered_arguments.push(self.visit_expr(arg)?);
                }

                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::FunctionCall {
                        function: Box::new(self.visit_expr(*function)?),
                        arguments: lowered_arguments,
                    }
                ))
            },
            AstExprKind::ArrayLiteral(items) => {
                let mut lowered_items = Vec::with_capacity(items.len());
                for item in items {
                    lowered_items.push(self.visit_expr(item)?);
                }

                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::ArrayLiteral(lowered_items)
                ))
            },
            AstExprKind::BinaryOp { left, operation, right} => {
                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::BinaryOp {
                        left: Box::new(self.visit_expr(*left)?),
                        operation,
                        right: Box::new(self.visit_expr(*right)?),
                    }
                ))
            },
            AstExprKind::BlockExpression(block) => {
                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::BlockExpression(Box::new(self.visit_block(*block)?))
                ))
            },
            AstExprKind::IfExpression { condition, body, else_statements } => {
                let mut lowered_else_statements = Vec::with_capacity(else_statements.len());
                for else_statement in else_statements {
                    let lowered_condition = if else_statement.0.is_some() {Some(Box::new(self.visit_expr(*else_statement.0.unwrap())?)) }
                        else { None };

                    lowered_else_statements.push((lowered_condition, Box::new(self.visit_block(*else_statement.1)?)));
                }

                Ok(HirExpr::new(
                    node.span,
                    HirExprKind::IfExpression {
                        condition: Box::new(self.visit_expr(*condition)?),
                        body: Box::new(self.visit_block(*body)?),
                        else_statements: lowered_else_statements,
                    }
                ))
            }
        }
    }

    fn visit_item(&mut self, node: AstItem<'a>) -> Result<HirItem<'a>, CompilerError<'a>> {
        todo!()
    }

    fn visit_block(&mut self, node: AstBlock<'a>) -> Result<HirBlock<'a>, CompilerError<'a>> {
        todo!()
    }



    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: &str) -> SymbolId {
        let id = SymbolId(uid::Id::<u16>::new());
        self.scopes.last_mut().unwrap().insert(name.to_string(), id).unwrap()
    }

    fn get_var_id(&self, name: &str) -> Option<SymbolId> {
        self.scopes.last()?.get(name).cloned()
    }
}