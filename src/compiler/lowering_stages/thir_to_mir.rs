use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::HirWrittenField;
use std::collections::HashMap;


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
        self.curr_size += size as u32;
        self.curr_size
    }

    fn address_of_var(&self, symbol: &SymbolId) -> u32 {
        self.addresses.get(symbol).copied().unwrap()
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

    fn lower_function(&mut self, id: &SymbolId, fields: &Vec<HirWrittenField>, body: &ThirBlock) -> MirBlock {
        self.scopes.push(Scope::new());
        let mut statements = Vec::new();

        for stmt in &body.statements {
            statements.extend(self.lower_stmt(stmt));
        }

        let size = self.scopes.pop().unwrap().curr_size;
        MirBlock {
            label: format!("function_{}", id),
            statements,
            size
        }
    }

    fn lower_stmt(&mut self, stmt: &ThirStmt) -> Vec<MirStmt> {

        match &stmt.kind {
            ThirStmtKind::Item(item) => {
                match &item.kind {
                    ThirItemKind::FunctionDeclaration { id, arguments, body } => {
                        vec![MirStmt::Block(Box::new(self.lower_function(id, arguments, &body) )) ]
                    }
                }
            },
            ThirStmtKind::VariableDeclaration { symbol_id, value, ty } => {
                let addr = self.scopes.last_mut().unwrap().assign_var_addr(symbol_id.clone(), self.type_table.get_type_size(&ty.id).unwrap());

                let Some(initial_value) = value else { return vec![]; };
                let value = self.lower_expr(initial_value);

                let mut instructions = Vec::new();

                instructions.extend(value.0);

                let addr_reg = TempReg::new();
                instructions.push(MirInstruction::Mov {
                    target: addr_reg,
                    value: addr,
                });

                let mut statements = Vec::new();

                for instr in instructions {
                    statements.push(MirStmt::Instruction(instr) );
                }




                statements.push(MirStmt::Instruction( MirInstruction::Store {
                        addr: MirValue::Load(Place::TempReg(addr_reg)),
                        value: value.1
                    })
                );

                statements
            },
            ThirStmtKind::Assignment { target, value } => {
                let value = self.lower_expr(value);
                let target_place = self.generate_place(target);

                let mut instructions = Vec::new();
                instructions.extend(target_place.0);
                instructions.extend(value.0);

                instructions.push(MirInstruction::Store {
                    addr: target_place.1,
                    value: value.1
                });

                wrap_as_statements(instructions)
            },
            ThirStmtKind::Semi(expr) => {
                wrap_as_statements(self.lower_expr(expr).0)
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
                MirValue::Number(*num as u32)
            },
            ThirExprKind::BinaryOp { left, operation, right } => {
                let left = self.lower_expr(&*left);
                let right = self.lower_expr(&*right);

                let result = TempReg::new();

                instructions.extend(left.0);
                instructions.extend(right.0);
                instructions.push(MirInstruction::Operation {
                        target: result,
                        val1: left.1,
                        val2: right.1,
                        op: operation.clone(),
                    }
                );

                MirValue::Load(Place::TempReg(result))
            },
            _ => {
                println!("skipping expr lowering: {}", expr.kind);
                MirValue::Number(0)
            }
        };

        (instructions, value)
    }

    fn generate_place(&mut self, expr: &ThirExpr) -> (Vec<MirInstruction>, Place) {
        match &expr.kind {
            ThirExprKind::Dereference(expr) => {
                let addr = self.lower_expr(&expr);

                let mut instructions = Vec::new();
                instructions.extend(addr.0);

                let temp_reg = self.ensure_in_register(addr.1);

                let place = Place::Address(temp_reg);

                (instructions, place)
            },
            ThirExprKind::Identifier(symbol) => {
                let offset = self.scopes.last().unwrap().address_of_var(symbol);
                let place = Place::FrameOffset(offset as usize);

                (Vec::new(), place)
            }
            _ => panic!("not implemented {}", expr),
        }
    }

    fn ensure_in_register(&mut self, value: MirValue) -> (Vec<MirInstruction>, TempReg) {
        let mut instructions = Vec::new();
        let temp_reg = TempReg::new();
        match value {
            MirValue::Number(num) => {
                instructions.push(MirInstruction::Mov {
                    target: temp_reg,
                    value: num
                } );
            },
            MirValue::Load(addr_reg) => {
                instructions.push( MirInstruction::Load {
                    target: temp_reg,
                    addr: addr_reg
                })
            }
        }
        (instructions, temp_reg)
    }
}

fn wrap_as_statements(instructions: Vec<MirInstruction>) -> Vec<MirStmt> {
    let mut statements = Vec::with_capacity(instructions.len());
    for instr in instructions {
        statements.push(MirStmt::Instruction(instr) );
    }
    statements
}
