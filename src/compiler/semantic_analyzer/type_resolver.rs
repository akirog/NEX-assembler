use std::collections::HashMap;
use std::env::var;
use crate::compiler::*;

pub struct TypeResolver<'a> {
    pub scope_handler: &'a ScopeHandler,
    pub symbol_table: &'a HashMap<NodeId, SymbolId>,
    pub declarations: &'a Vec<DeclNode<'a>>,
    pub expr_type_table: HashMap<NodeId, ResolvedType>,
}

impl<'a> TypeResolver<'a> {
    pub fn new(
        declarations: &'a Vec<DeclNode<'a>>,
        scope_handler: &'a ScopeHandler,
        symbol_table: &'a HashMap<NodeId, SymbolId>
    ) -> Self {
        Self {
            scope_handler,
            symbol_table,
            declarations,
            expr_type_table: HashMap::new(),
        }
    }

    pub fn resolve(&mut self) -> Result<&HashMap<NodeId, ResolvedType>, CompilerError<'a>> {

        for decl_node in self.declarations {
            match &decl_node.kind {
                DeclKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                    self.resolve_body(body)?;
                }
                _ => { /* Other declarations are ignored */ }
            }
        }

        Ok(&self.expr_type_table)
    }

    fn resolve_body(&mut self, body: &'a BlockNode<'a>) -> Result<(), CompilerError<'a>> {

        for stmt in &body.statements {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &'a StmtNode<'a>) -> Result<(), CompilerError<'a>> {

        match &stmt.kind {
            StmtKind::Expression(expr) => {
                self.resolve_expr(&expr)?;
            },
            StmtKind::Assignment { target, value } => {
                self.resolve_expr(&target)?;
                self.resolve_expr(&value)?;
            },
            StmtKind::VariableDeclaration { name, value, var_type} => {
                if let Some(value) = value {
                    self.resolve_expr(&value)?;
                }
            },

            _ => {}
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &'a ExprNode<'a>) -> Result<(), CompilerError<'a>> {

        match expr.kind {
            ExprKind::Identifier(_) => {
                let symbol_id = self.symbol_table.get(&expr.id).unwrap();
                let symbol = self.scope_handler.get_symbol(*symbol_id).unwrap();
                self.expr_type_table.insert(expr.id, symbol.var_type.clone());
            },

            _ => {}
        };

        Ok(())
    }
}