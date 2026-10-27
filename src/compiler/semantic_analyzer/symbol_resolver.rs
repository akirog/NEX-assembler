use std::collections::HashMap;
use crate::compiler::NodeId;
use crate::compiler::*;

pub struct SymbolResolver<'a> {
    declarations: &'a Vec<DeclNode<'a>>,
    scope_handler: ScopeHandler,
    symbol_table: HashMap<NodeId, SymbolId>,
}

impl<'a> SymbolResolver<'a> {
    pub fn new(declarations: &'a Vec<DeclNode<'a>>, scope_handler: ScopeHandler) -> Self {
        SymbolResolver {
            declarations,
            scope_handler,
            symbol_table: HashMap::new(),
        }
    }

    pub fn resolve(&mut self) -> Result<HashMap<NodeId, SymbolId>, CompilerError<'a>> {

        for decl in self.declarations {
            match &decl.kind {
                DeclKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                    self.resolve_body(body)?;
                },
                _ => { continue; }
            }
        }

        Ok(self.symbol_table.clone())
    }

    /// Resolve the symbols in a block node
    fn resolve_body(
        &mut self,
        body: &'a BlockNode<'a>,
    ) -> Result<(), CompilerError<'a>> {
        self.scope_handler.enter_scope(body.id);

        for stmt in &body.statements {
            self.resolve_statement(stmt)?;
        }

        self.scope_handler.pop_scope();
        Ok(())
    }

    /// Resolve a statements symbols
    fn resolve_statement(
        &mut self,
        stmt: &'a StmtNode<'a>,
    ) -> Result<(), CompilerError<'a>> {
        match &stmt.kind {
            StmtKind::Assignment { target, value } => {
                self.resolve_expression(target)?;
                self.resolve_expression(value)?;
            },
            StmtKind::VariableDeclaration { name, value, var_type: _var_type } => {
                if let Some(value) = value {
                    self.resolve_expression(value)?;
                }
            }
            StmtKind::Expression(expr) => {
                self.resolve_expression(expr)?;
            },

            // Inner scopes
            StmtKind::IfStatement { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_body(body)?;
            },
            StmtKind::ForLoop { init, condition, update, body} => {
                // these use the for loops local scope
                self.scope_handler.enter_scope(body.id);
                self.resolve_statement(init)?;
                self.resolve_expression(condition)?;
                self.resolve_statement(update)?;
                self.scope_handler.pop_scope();

                self.resolve_body(body)?;
            },
            StmtKind::WhileLoop { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_body(body)?;
            },
            StmtKind::Loop { body } => {
                self.resolve_body(body)?;
            }
            _ => { }
        };

        Ok(())
    }

    /// Resolve the symbols in an expression
    fn resolve_expression(
        &mut self,
        expr: &'a ExprNode<'a>,
    ) -> Result<(), CompilerError<'a>> {

        match &expr.kind {
            ExprKind::Identifier(name) => {
                let symbol_id = self.scope_handler.lookup_symbol(name);
                if let Some(symbol_id) = symbol_id {
                    self.symbol_table.insert(expr.id, symbol_id);
                } else {
                    return Err(CompilerError::new(
                        expr.span,
                        "Invalid identifier"
                    ))
                }
            },
            ExprKind::BinaryOp { left, operation, right } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            },
            ExprKind::AddressOf(name) => {
                let symbol_id = self.scope_handler.lookup_symbol(name);
                if let Some(symbol_id) = symbol_id {
                    self.symbol_table.insert(expr.id, symbol_id);
                } else {
                    return Err(CompilerError::new(
                        expr.span,
                        "Invalid identifier"
                    ))
                }
            },
            ExprKind::Dereference(expr) => {
                self.resolve_expression(expr)?;
            },
            ExprKind::TypeCast { new_type, expr } => {
                self.resolve_expression(expr)?;
            },
            ExprKind::FunctionCall { function, arguments } => {
                self.resolve_expression(function)?;
                for argument in arguments {
                    self.resolve_expression(argument)?;
                }
            },
            ExprKind::IndexAccess { left, index } => {
                self.resolve_expression(left)?;
                self.resolve_expression(index)?;
            },
            ExprKind::ArrayLiteral(values) => {
                for value in values {
                    self.resolve_expression(value)?;
                }
            },
            ExprKind::Number(_) | ExprKind::Char(_) | ExprKind::Bool(_) | ExprKind::String(_) => { }
        }

        Ok(())
    }
}