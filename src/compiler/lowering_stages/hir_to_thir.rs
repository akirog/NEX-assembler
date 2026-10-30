use crate::compiler::errors::CompilerError;
use crate::compiler::ir::hir::*;
use crate::compiler::ir::thir::*;

pub struct HirLowerer {

}

impl<'a> HirLowerer {
    pub fn new() -> Self {
        Self {

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
        Ok(Box::new(self.lower_expr(expr)?))
    }

    fn lower_expr(&mut self, expr: HirExpr<'a>) -> Result<ThirExpr<'a>, CompilerError<'a>> {
        let kind = match expr.kind {
            HirExprKind::String(string) => {
                ThirExprKind::String(string)
            },
            HirExprKind::Char(char) => {
                ThirExprKind::Char(char)
            },
            HirExprKind::Bool(value) => {
                ThirExprKind::Bool(value)
            },
            HirExprKind::Number(value) => {

            }

            _ => { todo!("{:?}", expr.kind); },
        };

        Ok(ThirExpr::new(expr.span, kind))
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
}