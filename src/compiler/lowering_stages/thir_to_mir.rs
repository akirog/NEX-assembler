use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::operations::Operation;
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::{HirWrittenField, TypeId, TypeKind};
use crate::compiler::ir::types::type_table::TypeTable;
use std::cmp::max;
use std::collections::HashMap;

#[derive(Default)]
pub struct ThirLowerer {
    type_table: TypeTable,
    local_vars: Vec<Vec<SymbolId>>,
    var_offsets: HashMap<SymbolId, u32>,
    frame_size: u32,
    total_frame_size: u32,
    functions: Vec<SymbolId>,
    label_count: u32,
    loop_label_stack: Vec<String>,
    loop_break_stack: Vec<String>,
}

impl ThirLowerer {
    pub fn new(type_table: TypeTable) -> Self {
        Self {
            type_table,
            .. Default::default()
        }
    }

    pub fn lower(&mut self, thir: Vec<ThirStmt>) -> Vec<MirStmt> {
        let mut mir = Vec::new();

        for stmt in &thir {
            match &stmt.kind {
                ThirStmtKind::Item(item) => {
                    match &item.kind {
                        ThirItemKind::FunctionDeclaration { id, arguments, body } => {
                            self.functions.push(id.clone());
                        }
                    }
                },
                _ => {}
            }
        }

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
        self.enter_scope();

        let mut args = Vec::new();
        for field in fields {
            self.local_vars.last_mut().unwrap().push(field.id.clone());
            self.frame_size += self.type_table.get_type_size(&self.type_table.get_symbol_type_id(&field.id).unwrap()).unwrap() as u32;
            self.var_offsets.insert(field.id.clone(), self.frame_size);
            args.push(self.frame_size);
        }

        let instructions = self.lower_block(body);
        self.pop_scope();

        let size = self.total_frame_size;
        self.total_frame_size = 0;

        MirBlock {
            label: format!("function_{}", id),
            instructions,
            size,
            args
        }
    }

    fn enter_scope(&mut self) {
        self.local_vars.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        let symbols = self.local_vars.pop().unwrap();

        for id in &symbols {
            let size = self.type_table.get_type_size(&self.type_table.get_symbol_type_id(id).unwrap()).unwrap() as u32;
            self.frame_size -= size;
        }
    }

    fn lower_block(&mut self, block: &ThirBlock) -> Vec<MirInstruction> {
        let mut instructions = Vec::new();

        for stmt in &block.statements {
            instructions.extend(self.lower_stmt(stmt));
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
                self.total_frame_size = max(self.total_frame_size, self.frame_size);
                self.local_vars.last_mut().unwrap().push(symbol_id.clone());
                self.var_offsets.insert(symbol_id.clone(), addr);

                let Some(initial_value) = value else { return vec![]; };

                let mut instructions = Vec::new();
                let value = self.lower_expr(initial_value, &mut instructions);

                let addr = Address::FrameOffset(addr as usize);
            
                self.copy_value(&addr, &value, &ty.id, &mut instructions);

                instructions
            },
            ThirStmtKind::Assignment { target, value, ty } => {
                let mut instructions = Vec::new();
                let value = self.lower_expr(value, &mut instructions);
                let target_addr = self.generate_expr_address(target, &mut instructions);

                self.copy_value(&target_addr, &value, &ty.id, &mut instructions);

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
        }
    }

    fn lower_expr(&mut self, expr: &ThirExpr, instructions: &mut Vec<MirInstruction>) -> MirValue {
        let value = match &expr.kind {
            ThirExprKind::Number(num) => {
                MirValue::Const(Constant::Number(*num as u32))
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
                    val2: MirValue::Const(Constant::Number(0)),
                    cond: MirCondition::Eq
                });
                self.enter_scope();
                let block_instructions = self.lower_block(body);
                self.pop_scope();

                instructions.push(MirInstruction::Indented { instructions: block_instructions });

                instructions.push(MirInstruction::Label { label: end_label });

                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::Bool(value) => {
                MirValue::Const(Constant::Number(if *value { 1 } else { 0 }))
            },
            ThirExprKind::Char(value) => {
                MirValue::Const(Constant::Number(*value as u32))
            },
            ThirExprKind::String(value) => {
                let mut values = Vec::with_capacity(value.len());

                for c in value.chars() {
                    values.push(MirValue::Const(Constant::Number(c as u32)));
                }

                MirValue::Array(values)
            },
            ThirExprKind::Identifier(id) => {
                if self.functions.contains(id) {
                    return MirValue::Const(Constant::Label(format!("function_{}", id)))
                }

                MirValue::Load(Address::FrameOffset(self.get_addr_of_var(id) as usize))
            },
            ThirExprKind::FunctionCall { function, arguments } => {
                let mut lowered_args = Vec::new();
                for arg in arguments {
                    lowered_args.push(self.lower_expr(arg, instructions));
                }

                let function = self.lower_expr(function, instructions);

                instructions.push(MirInstruction::Call {
                    dest: function,
                    args: lowered_args
                });

                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::Loop { body } => {
                let start_label = self.get_temp_label();
                let end_label = self.get_temp_label();

                self.loop_label_stack.push(start_label.clone());
                self.loop_break_stack.push(end_label.clone());
                instructions.push(MirInstruction::Label { label: start_label.clone() });

                self.enter_scope();
                instructions.extend(self.lower_block(body));
                self.pop_scope();

                instructions.push(MirInstruction::Jump { label: start_label });
                instructions.push(MirInstruction::Label { label: end_label });
                self.loop_label_stack.pop();
                self.loop_break_stack.pop();


                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::BlockExpression(block) => {
                self.enter_scope();
                let block_instructions = self.lower_block(block);
                self.pop_scope();

                instructions.push(MirInstruction::Indented { instructions: block_instructions });

                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::IndexAccess { left, index } => {
                let left_addr = self.generate_expr_address(left, instructions);
                let index = self.lower_expr(index, instructions);

                let mut left_value = match left_addr {
                    Address::Absolute(reg) => MirValue::Register(reg),
                    Address::FrameOffset(offset) => MirValue::FrameOffsetAddress(offset),
                };

                // When indexing a slice, we load the value at slice
                if matches!(self.type_table.get_type(&left.ty.id), TypeKind::Slice(_)) {
                    let reg = TempReg::new();

                    instructions.push(MirInstruction::Load {
                        target: reg,
                        addr: left_addr
                    });

                    left_value = MirValue::Register(reg);
                }

                let index_reg = TempReg::new();
                instructions.push(MirInstruction::Operation {
                    target: index_reg,
                    val1: index,
                    val2: MirValue::Const(Constant::Number(self.type_table.get_type_size(&self.type_table.dereference_type(&left.ty.id).unwrap()).unwrap() as u32)),
                    op: Operation::Multiply,
                });

                let addr_reg = TempReg::new();
                instructions.push(MirInstruction::Operation {
                    target: addr_reg,
                    val1: left_value,
                    val2: MirValue::Register(index_reg),
                    op: Operation::Add,
                });

                let reg = TempReg::new();
                instructions.push(MirInstruction::Load {
                    target: reg,
                    addr: Address::Absolute(addr_reg)
                });

                MirValue::Register(reg)
            },
            ThirExprKind::AddressOf(var) => {
                let addr = self.get_addr_of_var(var);

                MirValue::FrameOffsetAddress(addr as usize)
            },
            ThirExprKind::Dereference(expr) => {
                let addr = self.lower_expr(expr, instructions);

                MirValue::Load(Address::Absolute(self.ensure_in_register(addr, instructions)))
            },
            _ => {
                panic!("Not implemented: {:?}", expr.kind);
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
            MirValue::Const(constant) => {
                let temp_reg = TempReg::new();

                instructions.push(MirInstruction::Mov {
                    target: temp_reg,
                    value: constant
                });

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
            },
            MirValue::Array(values) => {
                panic!("Cannot ensure array in register")
            },
            MirValue::FrameOffsetAddress(addr) => {
                panic!("No no no dont do that!!")
            }
        };
        reg
    }

    fn copy_value(&mut self, destination: &Address, value: &MirValue, ty: &TypeId, instructions: &mut Vec<MirInstruction>) {
        match value {
            MirValue::Array(values) => {
                let mut curr_addr = destination.clone();
                let pointee_type = self.type_table.dereference_type(&ty).unwrap();
                let pointee_size = self.type_table.get_type_size(&pointee_type).unwrap();

                for value in values {
                    self.copy_value(&curr_addr, value, &pointee_type, instructions);

                    match curr_addr {
                        Address::FrameOffset(addr) => {
                            curr_addr = Address::FrameOffset(addr - pointee_size);
                        },
                        _ => { panic!("oh god no") }
                    }
                }
            },
            _ => {
                let size = self.type_table.get_type_size(ty).unwrap();
                instructions.push(MirInstruction::Store {
                    target: destination.clone(),
                    value: value.clone(),
                    byte: size == 1
                });
            }
        }
    }

    fn get_temp_label(&mut self) -> String {
        self.label_count += 1;
        format!("temp_label_{}", self.label_count - 1)
    }

    fn get_addr_of_var(&self, symbol: &SymbolId) -> u32 {
        self.var_offsets[symbol]
    }
}
