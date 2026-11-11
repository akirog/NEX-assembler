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
                        ThirItemKind::FunctionDeclaration { id, .. } => {
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
            self.frame_size += self.type_table.get_type_size(&self.type_table.get_symbol_type_id(&field.id).unwrap()) as u32;
            self.total_frame_size = self.frame_size;
            self.var_offsets.insert(field.id.clone(), self.frame_size);
            args.push(self.frame_size);
        }

        let instructions = self.lower_block(body);
        self.pop_scope();

        let size = self.total_frame_size;
        self.total_frame_size = 0;
        self.frame_size = 0;

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
            let size = self.type_table.get_type_size(&self.type_table.get_symbol_type_id(id).unwrap()) as u32;
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
            ThirStmtKind::Item(_) => {
                panic!("Found item at non global scope, not handled yet")
            },
            ThirStmtKind::VariableDeclaration { symbol_id, value, ty } => {
                self.frame_size += self.type_table.get_type_size(&ty.id) as u32;
                let addr = self.frame_size;
                self.total_frame_size = max(self.total_frame_size, self.frame_size);
                self.local_vars.last_mut().unwrap().push(symbol_id.clone());
                self.var_offsets.insert(symbol_id.clone(), addr);

                let Some(initial_value) = value else { return vec![]; };
                let addr = Address::FrameOffset(addr as usize);

                let mut instructions = Vec::new();
                self.lower_expr_into(initial_value, &addr, &ty.id, &mut instructions);

                instructions
            },
            ThirStmtKind::Assignment { target, value, ty } => {
                let mut instructions = Vec::new();
                let target_addr = self.generate_expr_address(target, &mut instructions);

                self.lower_expr_into(value, &target_addr, &ty.id, &mut instructions);

                instructions
            },
            ThirStmtKind::Return { value, .. } => {
                let mut instructions = Vec::new();
                
                let ret_instruction = MirInstruction::Ret {
                    value: if let Some(value) = value { self.lower_expr(value, &mut instructions) } else { MirValue::Const(Constant::Number(0)) }
                };

                instructions.push(ret_instruction);

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

    fn lower_expr_into(&mut self, expr: &ThirExpr, address: &Address, ty: &TypeId, instructions: &mut Vec<MirInstruction>) {
        if self.type_table.is_aggregate(ty) {
            let mut curr_addr = address.clone();

            match &expr.kind {
                ThirExprKind::ArrayLiteral(values) => {
                    let pointee_type = self.type_table.dereference_type(&ty).unwrap();
                    let pointee_size = self.type_table.get_type_size(&pointee_type);

                    for value in values {
                        self.lower_expr_into(value, &curr_addr, &pointee_type, instructions);

                        match curr_addr {
                            Address::FrameOffset(addr) => {
                                curr_addr = Address::FrameOffset(addr - pointee_size);
                            },
                            _ => { panic!("oh god no") }
                        }
                    }
                },
                ThirExprKind::String(string) => {
                    const CHARACTER_SIZE: usize = 1;
                    let pointee_size = CHARACTER_SIZE; // Character always has size of 1

                    for value in string.chars() {
                        instructions.push(MirInstruction::Store {
                            target: curr_addr.clone(),
                            value: MirValue::Const(Constant::Number(value as u32)),
                            byte: true
                        });

                        curr_addr = curr_addr.add(1, instructions);
                    }
                },
                ThirExprKind::StructLiteral { ty, values } => {
                    let struct_ty = self.type_table.get_type(ty);
                    let fields = match struct_ty {
                        TypeKind::Struct { fields } => fields,
                        _ => unreachable!()
                    };

                    for (i, value) in values.iter().enumerate() {
                        self.lower_expr_into(value, &curr_addr, &fields.types()[i], instructions);
                        let value_size = self.type_table.get_type_size(&fields.types()[i]);

                        curr_addr = curr_addr.add(1, instructions);
                    }
                }
                ThirExprKind::Identifier(id) => {
                    // For aggregate typed variables we just memory copy
                    let mut var_addr = self.generate_expr_address(expr, instructions);

                    for i in 0..self.type_table.get_type_size(&self.type_table.get_symbol_type_id(id).unwrap()) {
                        instructions.push(MirInstruction::Store {
                            target: curr_addr.clone(),
                            value: MirValue::Load(var_addr.clone()),
                            byte: true
                        });
                    }
                }
                _ => {
                    panic!("Unexpected expression for aggregate type: \n{}", expr.pretty_print(&self.type_table));
                }
            }
        } else {
            let value = self.lower_expr(expr, instructions);
            instructions.push(MirInstruction::Store {
                target: address.clone(),
                value,
                byte: self.type_table.get_type_size(ty) == 1
            })
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
                let mut next_label = self.get_temp_label();

                let cond_result = self.lower_expr(condition, instructions);

                // Skip if when not true
                instructions.push( MirInstruction::Branch {
                    dest: next_label.clone(),
                    val1: cond_result,
                    val2: MirValue::Const(Constant::Number(0)),
                    cond: MirCondition::Eq
                });
                self.enter_scope();
                let mut block_instructions = self.lower_block(body);
                self.pop_scope();
                block_instructions.push(MirInstruction::Jump { label: end_label.clone() });
                instructions.push(MirInstruction::Indented { instructions: block_instructions });

                for (i, (cond, body)) in else_statements.iter().enumerate() {
                    instructions.push(MirInstruction::Label { label: next_label.clone() });
                    next_label = self.get_temp_label();

                    let cond_result = if let Some(cond) = cond {
                        self.lower_expr(cond, instructions)
                    } else {
                        MirValue::Const(Constant::Number(1))
                    };

                    // Skip if when not true
                    instructions.push( MirInstruction::Branch {
                        dest: next_label.clone(),
                        val1: cond_result,
                        val2: MirValue::Const(Constant::Number(0)),
                        cond: MirCondition::Eq
                    });


                    self.enter_scope();
                    let mut block_instructions = self.lower_block(body);
                    self.pop_scope();

                    if i < else_statements.len() {
                        block_instructions.push(MirInstruction::Jump { label: end_label.clone() });
                    }

                    instructions.push(MirInstruction::Indented { instructions: block_instructions });
                }
                instructions.push(MirInstruction::Label { label: next_label.clone() });


                instructions.push(MirInstruction::Label { label: end_label });


                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::Bool(value) => {
                MirValue::Const(Constant::Number(if *value { 1 } else { 0 }))
            },
            ThirExprKind::Char(value) => {
                MirValue::Const(Constant::Number(*value as u32))
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
            ThirExprKind::IndexAccess { .. } => {
                let address = self.generate_expr_address(expr, instructions);

                let reg = TempReg::new();
                instructions.push(MirInstruction::Load {
                    target: reg,
                    addr: address
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
            ThirExprKind::Break { .. } => {
                instructions.push(MirInstruction::Jump {
                    label: self.loop_break_stack.last().unwrap().clone(),
                });

                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::Continue { .. } => {
                instructions.push(MirInstruction::Jump {
                    label: self.loop_label_stack.last().unwrap().clone(),
                });

                MirValue::Const(Constant::Number(0))
            },
            ThirExprKind::ArrayLiteral { .. } |
            ThirExprKind::StructLiteral { .. } => {
                unreachable!()
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
                    val2: MirValue::Const(Constant::Number(self.type_table.get_type_size(&self.type_table.dereference_type(&left.ty.id).unwrap()) as u32)),
                    op: Operation::Multiply,
                });

                let addr_reg = TempReg::new();
                instructions.push(MirInstruction::Operation {
                    target: addr_reg,
                    val1: left_value,
                    val2: MirValue::Register(index_reg),
                    op: Operation::Add,
                });

                Address::Absolute(addr_reg)
            }
            _ => panic!("not implemented {}", expr.pretty_print(&self.type_table)),
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
            MirValue::FrameOffsetAddress(_) => {
                panic!("No no no dont do that!!")
            }
        };
        reg
    }

    fn get_temp_label(&mut self) -> String {
        self.label_count += 1;
        format!("temp_label_{}", self.label_count - 1)
    }

    fn get_addr_of_var(&self, symbol: &SymbolId) -> u32 {
        self.var_offsets[symbol]
    }
}
