use std::collections::HashMap;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::types::type_table::TypeTable;


struct Scope {
    addresses: HashMap<SymbolId, u32>,
    curr_size: u32,
}

impl Scope {
    fn new() -> Self {
        Self {
            addresses: HashMap::new(),
            curr_size: 0,
        }
    }

    fn assign_var_addr(&mut self, symbol: SymbolId, size: usize) -> u32 {
        self.addresses.insert(symbol, self.curr_size);
        let addr = self.curr_size;
        self.curr_size += size as u32;
        addr
    }

    fn address_of_var(&self, symbol: SymbolId) -> u32 {
        self.addresses.get(&symbol).copied().unwrap()
    }
}



pub struct ThirLowerer {
    type_table: TypeTable,
    scopes: Vec<Scope>,
}

impl ThirLowerer {
    pub fn new(type_table: TypeTable) -> Self {
        Self {
            type_table,
            scopes: Vec::new(),
        }
    }

    pub fn lower(&mut self, thir: Vec<ThirStmt>) -> Vec<MirStmt> {
        let mut mir = Vec::new();

        for stmt in &thir {
            mir.extend(self.lower_stmt(stmt));
        }

        mir
    }

    fn lower_stmt(&mut self, stmt: &ThirStmt) -> Vec<MirStmt> {

        match &stmt.kind {
            ThirStmtKind::VariableDeclaration { symbol_id, value, ty } => {
                self.scopes.last_mut().unwrap().assign_var_addr(symbol_id.clone(), self.type_table.get_type_size(&ty.id).unwrap());

                if value.is_none() { return vec![]; };

                vec![]
            },
            ThirStmtKind::Assignment { target, value } => {
                let value = self.lower_expr(value);

                match &target.kind {
                    ThirExprKind::Dereference(expr) => {
                        let addr = self.lower_expr(&expr);

                        let mut instructions = Vec::new();

                        instructions.extend(value.0);
                        instructions.extend(addr.0);

                        let mut statements = Vec::new();

                        for instr in instructions {
                            statements.push(MirStmt { kind: MirStmtKind::Instruction(instr) } );
                        }

                        statements.push(MirStmt {
                            kind: MirStmtKind::Instruction( MirInstruction {
                                kind: MirInstructionKind::Store {
                                    addr: addr.1,
                                    value: value.1
                                }
                            })
                        });

                        statements
                    }
                    _ => panic!("not implemented {:?}", target.kind),
                }
            },
            ThirStmtKind::Semi(expr) => {
                let mut statements = Vec::new();
                for instr in self.lower_expr(expr).0 {
                    statements.push(MirStmt { kind: MirStmtKind::Instruction(instr) } );
                };
                statements
            },
            _ => {
                println!("skipping lowering statement: {}", stmt);
                vec![]
            }
        }
    }

    fn lower_expr(&mut self, expr: &ThirExpr) -> (Vec<MirInstruction>, MirValue) {
        let mut instructions: Vec<MirInstruction> = Vec::new();

        let value = match &expr.kind {
            ThirExprKind::Number(num) => {
                let reg = TempReg::new();
                instructions.push(MirInstruction { kind: MirInstructionKind::Mov { target: reg, value: *num as u32 }});
                MirValue {
                    kind: MirValueKind::TempReg(reg),
                }
            },
            ThirExprKind::BinaryOp { left, operation, right } => {
                let left = self.lower_expr(&*left);
                let right = self.lower_expr(&*right);

                let result = TempReg::new();

                instructions.extend(left.0);
                instructions.extend(right.0);
                instructions.push(MirInstruction {
                    kind: MirInstructionKind::Operation {
                        target: result,
                        val1: left.1,
                        val2: right.1,
                        op: operation.clone(),
                    }
                });

                MirValue {
                    kind: MirValueKind::TempReg(result)
                }
            },
            _ => {
                println!("skipping expr lowering: {}", expr.kind);
                MirValue {
                    kind: MirValueKind::Number(0)
                }
            }
        };

        (instructions, value)
    }
}
