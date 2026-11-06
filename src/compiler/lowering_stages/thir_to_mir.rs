use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::HirWrittenField;
use std::collections::HashMap;



pub struct ThirLowerer {
    type_table: TypeTable,
    local_vars: Vec<Vec<SymbolId>>,
    var_offsets: HashMap<SymbolId, u32>,
    frame_size: u32,

}

impl ThirLowerer {
    pub fn new(type_table: TypeTable) -> Self {
        Self {
            type_table,
            local_vars: Vec::new(),
            var_offsets: HashMap::new(),
            frame_size: 0,
        }
    }

    pub fn lower(&mut self, thir: Vec<ThirStmt>) -> Vec<MirStmt> {
        let mut mir = Vec::new();

        for stmt in &thir {
            match &stmt.kind {
                ThirStmtKind::Item(item) => {
                    match &item.kind {
                        ThirItemKind::FunctionDeclaration { id, arguments, body } => {
                            mir.push(MirStmt::Block(Box::new(self.lower_function(id, arguments, body))));
                        }
                    }
                },
                _ => panic!("Found non item at global scope")
            }
        }

        mir
    }

    fn lower_function(&mut self, id: &SymbolId, fields: &Vec<HirWrittenField>, body: &ThirBlock) -> MirBlock {
        let instructions = self.lower_block(body);

        MirBlock {
            label: format!("function_{}", id),
            instructions,
            size: self.frame_size,
        }
    }

    fn lower_block(&mut self, block: &ThirBlock) -> Vec<MirInstruction> {
        let mut instructions = Vec::new();

        self.local_vars.push(Vec::new());

        for stmt in &block.statements {
            instructions.extend(self.lower_stmt(stmt));
        }

        let symbols = self.local_vars.pop().unwrap();

        for id in &symbols {
            let size = self.type_table.get_type_size(&self.type_table.get_symbol_type_id(id).unwrap()).unwrap() as u32;
            self.frame_size -= size;
        }

        instructions
    }


    fn lower_stmt(&mut self, stmt: &ThirStmt) -> Vec<MirInstruction> {

        match &stmt.kind {
            ThirStmtKind::Item(item) => {
                panic!("Found item at non global scope, not handled yet")
            },
            ThirStmtKind::VariableDeclaration { symbol_id, value, ty } => {
                self.frame_size += self.type_table.get_type_size(&ty.id).unwrap() as u32;
                let addr = self.frame_size;
                self.local_vars.last_mut().unwrap().push(symbol_id.clone());
                self.var_offsets.insert(symbol_id.clone(), addr);

                let Some(initial_value) = value else { return vec![]; };

                let mut instructions = Vec::new();
                let value = self.lower_expr(initial_value, &mut instructions);

                let addr = Address::FrameOffset(addr as usize);

                instructions.push(MirInstruction::Store {
                        target: addr,
                        value
                });

                instructions
            },
            ThirStmtKind::Assignment { target, value } => {
                let mut instructions = Vec::new();
                let value = self.lower_expr(value, &mut instructions);
                let target_addr = self.generate_expr_address(target, &mut instructions);


                instructions.push(MirInstruction::Store {
                    target: target_addr,
                    value
                });


                instructions
            },
            ThirStmtKind::Semi(expr) => {
                let mut instructions = Vec::new();
                self.lower_expr(expr, &mut instructions);
                instructions
            },
            ThirStmtKind::Expression(expr) => {
                let mut instructions = Vec::new();
                self.lower_expr(expr, &mut instructions);
                instructions
            },
            _ => {
                println!("skipped lowering statement: {}", stmt);
                vec![]
            }
        }
    }

    fn lower_expr(&mut self, expr: &ThirExpr, instructions: &mut Vec<MirInstruction>) -> MirValue {
        let value = match &expr.kind {
            ThirExprKind::Number(num) => {
                MirValue::Number(*num as u32)
            },
            ThirExprKind::BinaryOp { left, operation, right } => {
                let left = self.lower_expr(&*left, instructions);
                let right = self.lower_expr(&*right, instructions);

                let result = TempReg::new();

                instructions.push(MirInstruction::Operation {
                        target: result,
                        val1: left,
                        val2: right,
                        op: operation.clone(),
                    }
                );

                MirValue::Register(result)
            },
            ThirExprKind::IfExpression { condition, body, else_statements } => {
                let end_label = self.get_temp_label();

                let cond_result = self.lower_expr(condition, instructions);

                // Skip if when not true
                instructions.push( MirInstruction::Branch {
                    dest: end_label.clone(),
                    val1: cond_result,
                    val2: MirValue::Number(0),
                    cond: MirCondition::Eq
                });

                let block_instructions = self.lower_block(body);
                instructions.push(MirInstruction::Indented { instructions: block_instructions });

                MirValue::Number(0)
            },
            ThirExprKind::Bool(value) => {
                MirValue::Number(if *value { 1 } else { 0 })
            },
            ThirExprKind::Char(value) => {
                MirValue::Number(*value as u32)
            },
            ThirExprKind::Identifier(id) => {
                MirValue::Load(Address::FrameOffset(self.get_addr_of_var(id) as usize))
            },
            _ => {
                println!("skipping expr lowering: {}", expr.kind);
                MirValue::Number(0)
            }
        };

        value
    }

    fn generate_expr_address(&mut self, expr: &ThirExpr, instructions: &mut Vec<MirInstruction>) -> Address {
        match &expr.kind {
            ThirExprKind::Dereference(expr) => {
                let addr = self.lower_expr(&expr, instructions);
                let temp_reg = self.ensure_in_register(addr, instructions);

                let place = Address::Absolute(temp_reg);

                place
            },
            ThirExprKind::Identifier(symbol) => {
                let offset = self.get_addr_of_var(symbol);
                let place = Address::FrameOffset(offset as usize);

                place
            }
            _ => panic!("not implemented {}", expr),
        }
    }

    fn ensure_in_register(&mut self, value: MirValue, instructions: &mut Vec<MirInstruction>) -> TempReg {
        let reg = match value {
            MirValue::Number(num) => {
                let temp_reg = TempReg::new();
                instructions.push(MirInstruction::Mov {
                    target: temp_reg,
                    value: num
                } );
                temp_reg
            },
            MirValue::Load(addr) => {
                let temp_reg = TempReg::new();
                instructions.push( MirInstruction::Load {
                    target: temp_reg,
                    addr
                });
                temp_reg
            },
            MirValue::Register(reg) => {
                reg
            }
        };
        reg
    }

    fn get_temp_label(&mut self) -> String {
        format!("temp_label_{}", uid::Id::<u32>::new())
    }

    fn get_addr_of_var(&self, symbol: &SymbolId) -> u32 {
        self.var_offsets[symbol]
    }
}
