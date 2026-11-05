use crate::compiler::assembly_generation::register_manager::RegisterManager;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::operations::*;

enum OperandKind {
    Reg(String)
}

struct Operand {
    kind: OperandKind
}

impl Operand {
    fn get_in_reg(&mut self) -> &str {
        match self.kind {
            OperandKind::Reg(ref reg) => reg,
        }
    }

    fn from_reg(reg: &str) -> Operand {
        Operand {
            kind: OperandKind::Reg(String::from(reg))
        }
    }
}


pub struct AssemblyGenerator {
    assembly: Vec<String>,
    reg_manager: RegisterManager,
}

impl AssemblyGenerator {
    pub fn new() -> Self {
        Self {
            assembly: Vec::new(),
            reg_manager: RegisterManager::new(),
        }
    }

    pub fn generate(&mut self, thir: Vec<ThirStmt>) -> &Vec<String> {
        for stmt in &thir {
            self.generate_stmt(stmt);
        }

        &self.assembly
    }

    fn generate_stmt(&mut self, stmt: &ThirStmt) {
        match &stmt.kind {
            ThirStmtKind::Semi(expr) => {
                self.generate_expr(&**expr);
            },
            ThirStmtKind::Expression(expr) => {
                self.generate_expr(&**expr);
            },
            _ => {
                println!("skipped {} generation", stmt);
            }
        }
    }

    /// Lowers an expression and returns an operand with the value
    fn generate_expr(&mut self, expr: &ThirExpr) -> Operand {
        match &expr.kind {
            ThirExprKind::BinaryOp { left, operation, right } => {
                let mut left = self.generate_expr(left);
                let mut right = self.generate_expr(right);

                let op_str = match operation {
                    Operation::Add => "add",
                    _ => {""}
                };

                let result_reg = self.reg_manager.get_temp_reg();

                let asm = format!("{} {}, {}, {} ; Binary operation, {}", op_str, result_reg, left.get_in_reg(), right.get_in_reg(), expr);
                self.assembly.push(asm);

                Operand::from_reg(result_reg)
            },
            ThirExprKind::Number(num) => {
                let result_reg = self.reg_manager.get_temp_reg();

                let asm = format!("mov {}, {}", result_reg, num);
                self.assembly.push(asm);

                Operand::from_reg(result_reg)
            },
            _ => {
                println!("skipped {} generation", expr);
                Operand::from_reg("")
            }
        }
    }
}