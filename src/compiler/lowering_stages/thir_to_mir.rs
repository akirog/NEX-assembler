use crate::compiler::assembly_generation::register_manager::RegisterManager;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::operations::{ImmOperation, Operation};
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::{TypeId, TypeKind};
use std::cmp::max;
use std::collections::HashMap;

pub struct ThirLowerer<'a> {
    type_table: &'a TypeTable,
    local_vars: Vec<Vec<SymbolId>>,
    var_offsets: HashMap<SymbolId, u32>,
    frame_size: u32,
    total_frame_size: u32,
    function_names: Vec<SymbolId>,
    reg_manager: RegisterManager,
    label_count: u32,
    loop_label_stack: Vec<String>,
    loop_break_stack: Vec<String>,
}

impl<'a> ThirLowerer<'a> {
    pub fn new(type_table: &'a TypeTable) -> Self {
        Self {
            type_table,
            local_vars: Vec::new(),
            var_offsets: HashMap::new(),
            frame_size: 0,
            total_frame_size: 0,
            function_names: Vec::new(),
            reg_manager: RegisterManager::new(),
            label_count: 0,
            loop_break_stack: Vec::new(),
            loop_label_stack: Vec::new(),
        }
    }

    pub fn lower(&mut self, thir: Vec<ThirStmt>) -> Vec<MirStmt> {
        let mut mir = Vec::new();

        for stmt in &thir {
            match &stmt.kind {
                ThirStmtKind::Item(item) => {
                    match &item.kind {
                        ThirItemKind::FunctionDeclaration { id, .. } => {
                            self.function_names.push(id.clone());
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
                        ThirItemKind::FunctionDeclaration { id, param_ids, ty, body } => {
                            mir.push(MirStmt::Block(Box::new(self.lower_function(id, param_ids, ty, body))));
                        }
                    }
                },
                _ => panic!("Found non item at global scope")
            }
        }

        mir
    }

    fn lower_function(&mut self, id: &SymbolId, param_ids: &Vec<SymbolId>, ty: &TypeId, body: &ThirBlock) -> MirBlock {
        self.enter_scope();
        let mut instructions = Vec::new();

        let TypeKind::Function { params: param_types, .. } = self.type_table.get_type(ty) else { panic!("Expected function type") };

        let mut destinations = Vec::with_capacity(param_types.len());
        for (param_type, id) in param_types.iter().zip(param_ids) {
            let param_size = self.type_table.get_type_size(param_type);
            self.frame_size += param_size as u32;

            self.local_vars.last_mut().unwrap().push(id.clone());
            self.var_offsets.insert(id.clone(), self.frame_size);

            destinations.push(self.frame_size);
        }
        self.total_frame_size = self.frame_size;


        let param_slots = self.get_function_param_slots(ty);
        let mut reg_arg_destinations = Vec::new();
        for (i, slot) in param_slots.iter().enumerate() {
            let dest: u32 = destinations[i];
            match slot {
                ParamSlot::ArgReg => {
                    reg_arg_destinations.push(dest);
                },
                ParamSlot::SpOffset(offset) => {
                    // Copy from sp offset to destination
                    let reg = self.reg_manager.get_temp_reg();
                    let mut j = 0;
                    while j < self.type_table.get_type_size(&param_types[i]) {
                        instructions.push(MirInstruction::Load {
                            target: reg,
                            addr: Address::BasePointerOffset(-(*offset as i32 + j as i32 + 8)),
                        });
                        let (inc, byte) = match (self.type_table.get_type_size(&param_types[i]) - j) % 4 {
                            0 => {
                                (4, false)
                            },
                            _ => (1, true)
                        };

                        instructions.push(MirInstruction::Store {
                            target: Address::BasePointerOffset((dest as usize - j) as i32),
                            value: reg,
                            byte,
                        });
                        j += inc;
                    }
                    self.reg_manager.free_temp_reg(reg)
                }
            }
        }

         instructions.extend(self.lower_block(body));
        self.pop_scope();

        let size = self.total_frame_size;
        self.total_frame_size = 0;
        self.frame_size = 0;

        MirBlock {
            label: format!("function_{}", id),
            instructions,
            size,
            reg_args: reg_arg_destinations
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
            instructions.push(MirInstruction::Comment("".to_string()));
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
                let addr = Address::BasePointerOffset(addr as i32);

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
                    value: if let Some(value) = value {
                        self.lower_expr(value, &mut instructions)
                            .ensure_in_reg(&mut instructions, &mut self.reg_manager)
                    } else {
                        MirValue::Const(Constant::Number(0))
                            .ensure_in_reg(&mut instructions, &mut self.reg_manager)
                    }
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
                        curr_addr.add(pointee_size as i32, instructions);
                    }
                },
                ThirExprKind::String(string) => {
                    const CHARACTER_SIZE: i32 = 1;

                    for value in string.chars() {
                        let reg = MirValue::Const(Constant::Number(value as u32)).ensure_in_reg(instructions, &mut self.reg_manager);

                        instructions.push(MirInstruction::Store {
                            target: curr_addr.clone(),
                            value: reg,
                            byte: true
                        });

                        self.reg_manager.free_temp_reg(reg);

                        curr_addr.add(CHARACTER_SIZE, instructions);
                    }
                },
                ThirExprKind::StructLiteral { ty, fields: given_values } => {
                    let layout = self.type_table.get_struct_layout(ty);

                    for (i, field) in given_values.iter().enumerate() {
                        self.lower_expr_into(&*field.value, &curr_addr, &layout.types[i], instructions);
                        curr_addr.add(layout.sizes[i] as i32, instructions);
                    }
                }
                ThirExprKind::Identifier(id) => {
                    // For aggregate typed variables we just memory copy
                    let mut var_addr = self.generate_expr_address(expr, instructions);

                    let size = self.type_table.get_type_size(&self.type_table.get_symbol_type_id(id).unwrap());
                    let reg = self.reg_manager.get_temp_reg();
                    let mut i = 0;
                    while i < size {
                        let (inc, byte) = match (size - i) % 4 {
                            0 => {
                                (4, false)
                            },
                            _ => (1, true)
                        };

                        instructions.push(MirInstruction::Load {
                            target: reg,
                            addr: var_addr.clone(),
                        });

                        instructions.push(MirInstruction::Store {
                            target: curr_addr.clone(),
                            value: reg,
                            byte
                        });


                        var_addr.add(inc, instructions);
                        curr_addr.add(inc, instructions);
                        i += inc as usize;
                    }
                    self.reg_manager.free_temp_reg(reg);
                    var_addr.free_if_reg(&mut self.reg_manager);
                },
                ThirExprKind::ImplicitCoercion { expr, new_type: _ } => {
                    let ThirExprKind::AddressOf(expr) = &expr.kind else { panic!("Unhandled implicit coercion") };

                    let expr_ty = self.type_table.get_type(&expr.ty.id);
                    let TypeKind::Array { ty: _, len } = expr_ty else {
                        panic!("Unexpected expression for aggregate type {}", self.type_table.type_as_string(&expr.ty.id))
                    };

                    // Store var addr at address + 0
                    let address = self.generate_expr_address(expr, instructions);
                    match address {
                        Address::BasePointerOffset(offset) => {
                            let reg = MirValue::FrameOffsetAddress(offset).ensure_in_reg(instructions, &mut self.reg_manager);
                            instructions.push(MirInstruction::Store {
                                target: curr_addr.clone(),
                                value: reg,
                                byte: false
                            });
                            self.reg_manager.free_temp_reg(reg);
                        },
                        _ => unimplemented!(),
                    }

                    // Store arr len at address + 4
                    let reg = self.reg_manager.get_temp_reg();
                    instructions.push(MirInstruction::Mov {
                        target: reg,
                        value: Constant::Number(len as u32)
                    });

                    curr_addr.add(4, instructions);
                    instructions.push(MirInstruction::Store {
                        target: curr_addr.clone(),
                        value: reg,
                        byte: false
                    });
                    self.reg_manager.free_temp_reg(reg);

                    address.free_if_reg(&mut self.reg_manager);
                },
                _ => {
                    println!("type: {:?}", self.type_table.get_type(&ty));
                    panic!("Unexpected expression for aggregate type: \n{}", expr.pretty_print(&self.type_table));
                }
            }

            curr_addr.free_if_reg(&mut self.reg_manager);
        } else {
            let size = self.type_table.get_type_size(ty);
            assert!(size == 1 || size == 4, "Unexpected variable size");
            let value = self.lower_expr(expr, instructions).ensure_in_reg(instructions, &mut self.reg_manager);
            instructions.push(MirInstruction::Store {
                target: address.clone(),
                value,
                byte: size == 1
            });
            self.reg_manager.free_temp_reg(value);
        }

    }

    fn lower_expr(&mut self, expr: &ThirExpr, instructions: &mut Vec<MirInstruction>) -> MirValue {
        let value = match &expr.kind {
            ThirExprKind::Number(num) => {
                MirValue::Const(Constant::Number(*num as u32))
            },
            ThirExprKind::BinaryOp { left, operation, right } => {
                let mut left = self.lower_expr(&*left, instructions);
                let mut right = self.lower_expr(&*right, instructions);

                let result = self.reg_manager.get_temp_reg();

                let instr = MirInstruction::Operation {
                    target: result,
                    left: left.ensure_in_reg(instructions, &mut self.reg_manager),
                    right: right.ensure_in_reg(instructions, &mut self.reg_manager),
                    op: operation.clone(),
                };

                instructions.push(instr);

                left.free_if_reg(&mut self.reg_manager);
                right.free_if_reg(&mut self.reg_manager);

                MirValue::Register(result)
            },
            ThirExprKind::IfExpression { condition, body, else_statements } => {
                let end_label = self.get_temp_label();
                let mut next_label = self.get_temp_label();

                let mut cond_result = self.lower_expr(condition, instructions);

                // Skip if when not true
                let mut zero = MirValue::Const(Constant::Number(0));
                let instr = MirInstruction::Branch {
                    dest: next_label.clone(),
                    val1: cond_result.ensure_in_reg(instructions, &mut self.reg_manager),
                    val2: zero.ensure_in_reg(instructions, &mut self.reg_manager),
                    cond: MirCondition::Eq
                };
                instructions.push( instr);
                cond_result.free_if_reg(&mut self.reg_manager);
                zero.free_if_reg(&mut self.reg_manager);


                self.enter_scope();
                let mut block_instructions = self.lower_block(body);
                self.pop_scope();
                block_instructions.push(MirInstruction::Jump { label: end_label.clone() });
                instructions.push(MirInstruction::Indented { instructions: block_instructions });

                for (i, (cond, body)) in else_statements.iter().enumerate() {
                    instructions.push(MirInstruction::Label { label: next_label.clone() });
                    next_label = self.get_temp_label();

                    let mut cond_result = if let Some(cond) = cond {
                        self.lower_expr(cond, instructions)
                    } else {
                        MirValue::Const(Constant::Number(1))
                    };

                    // Skip if when not true
                    let mut zero = MirValue::Const(Constant::Number(0));
                    let instr = MirInstruction::Branch {
                        dest: next_label.clone(),
                        val1: cond_result.ensure_in_reg(instructions, &mut self.reg_manager),
                        val2: zero.ensure_in_reg(instructions, &mut self.reg_manager),
                        cond: MirCondition::Eq
                    };
                    instructions.push( instr);
                    cond_result.free_if_reg(&mut self.reg_manager);
                    zero.free_if_reg(&mut self.reg_manager);


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
                if self.function_names.contains(id) {
                    return MirValue::Const(Constant::Label(format!("function_{}", id)))
                }

                if self.type_table.is_aggregate(&self.type_table.get_symbol_type_id(id).unwrap()) {
                    panic!("Aggregate in expression");
                }

                MirValue::Load(self.get_addr_of_var(id))
            },
            ThirExprKind::FunctionCall { function, arguments } => {
                // Find out how much to increment sp by for overflow args
                let TypeKind::Function { params: param_types, .. } = self.type_table.get_type(&function.ty.id)
                else { panic!("Expected function type for function call") };

                let mut total_sp_increment = 0;
                for param_type in param_types {
                    if self.type_table.is_aggregate(&param_type) {
                        total_sp_increment += self.type_table.get_type_size(&param_type);
                    }
                }

                instructions.push(MirInstruction::AsmText(
                    format!("subi sp, sp, {total_sp_increment}")
                ));

                // Put arguments in their slots
                let arg_slots = self.get_function_param_slots(&function.ty.id);
                let mut reg_arg_locations = Vec::new();
                for (arg, slot) in arguments.iter().zip(arg_slots) {
                    match slot {
                        ParamSlot::ArgReg => {
                            let arg_reg = self.lower_expr(arg, instructions).ensure_in_reg(instructions, &mut self.reg_manager);
                            reg_arg_locations.push(arg_reg);
                        },
                        ParamSlot::SpOffset(offset) => {
                            self.lower_expr_into(arg, &Address::StackPointerOffset(offset as usize), &arg.ty.id, instructions);
                        }
                    }
                }

                let mut function = self.lower_expr(function, instructions);

                let instr = MirInstruction::Call {
                    dest: function.ensure_in_reg(instructions, &mut self.reg_manager),
                    reg_args: reg_arg_locations.clone(),
                };
                instructions.push(instr);
                function.free_if_reg(&mut self.reg_manager);
                
                for reg in reg_arg_locations {
                    self.reg_manager.free_temp_reg(reg);
                };

                instructions.push(MirInstruction::AsmText(
                    format!("addi sp, sp, {total_sp_increment}")
                ));

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

                let reg = self.reg_manager.get_temp_reg();

                instructions.push(MirInstruction::Load {
                    target: reg,
                    addr: address.clone()
                });

                address.free_if_reg(&mut self.reg_manager);

                MirValue::Register(reg)
            },
            ThirExprKind::AddressOf(var) => {
                let addr = self.generate_expr_address(var, instructions);

                match addr {
                    Address::Absolute(reg) => MirValue::Register(reg),
                    Address::BasePointerOffset(offset) => MirValue::FrameOffsetAddress(offset),
                    Address::StackPointerOffset(offset) => MirValue::StackPointerOffset(offset),
                }
            },
            ThirExprKind::Dereference(expr) => {
                let mut addr = self.lower_expr(expr, instructions);

                MirValue::Load(Address::Absolute(addr.ensure_in_reg(instructions, &mut self.reg_manager)))
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
            ThirExprKind::MemberAccess { .. } => {
                let addr = self.generate_expr_address(expr, instructions);

                let reg = self.reg_manager.get_temp_reg();

                instructions.push(MirInstruction::Load {
                    target: reg,
                    addr: addr.clone()
                });

                addr.free_if_reg(&mut self.reg_manager);

                MirValue::Register(reg)
            },
            ThirExprKind::ArrayLiteral { .. } |
            ThirExprKind::StructLiteral { .. } => {
                unreachable!()
            },
            ThirExprKind::ImplicitCoercion { .. } => {
                unreachable!()
            }
            _ => {
                unimplemented!("Not implemented: {}", expr.pretty_print(&self.type_table));
            }
        };

        value
    }

    fn generate_expr_address(&mut self, expr: &ThirExpr, instructions: &mut Vec<MirInstruction>) -> Address {
        match &expr.kind {
            ThirExprKind::Dereference(expr) => {
                let mut addr = self.lower_expr(&expr, instructions);
                let temp_reg = addr.ensure_in_reg(instructions, &mut self.reg_manager);

                Address::Absolute(temp_reg)
            },
            ThirExprKind::Identifier(symbol) => {
                self.get_addr_of_var(symbol)
            },
            ThirExprKind::IndexAccess { left, index } => {
                let left_addr = self.generate_expr_address(left, instructions);
                let index = self.lower_expr(index, instructions)
                    .ensure_in_reg(instructions, &mut self.reg_manager);


                let mut left_value = match left_addr {
                    Address::Absolute(reg) => MirValue::Register(reg),
                    Address::BasePointerOffset(offset) => MirValue::FrameOffsetAddress(offset),
                    Address::StackPointerOffset(offset) => MirValue::StackPointerOffset(offset),
                }.ensure_in_reg(instructions, &mut self.reg_manager);


                // When indexing a slice, we load the value at slice
                if matches!(self.type_table.get_type(&left.ty.id), TypeKind::Slice(_)) {
                    let reg = self.reg_manager.get_temp_reg();

                    instructions.push(MirInstruction::Load {
                        target: reg,
                        addr: left_addr.clone()
                    });

                    self.reg_manager.free_temp_reg(left_value);
                    left_value = reg;
                }


                let index_reg = self.reg_manager.get_temp_reg();
                instructions.push(MirInstruction::ImmOperation {
                    target: index_reg,
                    left: index,
                    right: Constant::Number(
                            self.type_table.get_type_size(&self.type_table.dereference_type(&left.ty.id).unwrap()) as u32
                    ),
                    op: ImmOperation::Multiply
                });

                let addr_reg = self.reg_manager.get_temp_reg();
                instructions.push(MirInstruction::Operation {
                    target: addr_reg,
                    left: left_value,
                    right: index_reg,
                    op: Operation::Add,
                });
                self.reg_manager.free_temp_reg(index_reg);

                Address::Absolute(addr_reg)
            },
            ThirExprKind::MemberAccess { left, member } => {
                let mut left_addr = self.generate_expr_address(&left, instructions);

                let Some(addr_add) = self.type_table.get_member_offset(&left.ty.id, member) else {
                    panic!("Could not get member offset");
                };

                left_addr.add(addr_add as i32, instructions);
                left_addr
            }
            _ => panic!("not implemented {}", expr.pretty_print(&self.type_table)),
        }
    }

    fn get_temp_label(&mut self) -> String {
        self.label_count += 1;
        format!("temp_label_{}", self.label_count - 1)
    }

    fn get_addr_of_var(&self, symbol: &SymbolId) -> Address {
        Address::BasePointerOffset(self.var_offsets[symbol] as i32)
    }

    fn get_function_param_slots(&self, fn_type: &TypeId) -> Vec<ParamSlot> {
        let arg_types = match self.type_table.get_type(fn_type) {
            TypeKind::Function { params, .. } => params,
            _ => panic!("Unexpected type given, not function type")
        };

        let mut sp_offset = 0;
        let mut arg_counter = 0;
        let arg_slots = arg_types.iter().map(|ty| {
            match self.type_table.is_aggregate(ty) || arg_counter > 4 {
                true => {
                    let offset = sp_offset;
                    sp_offset += self.type_table.get_type_size(ty);
                    ParamSlot::SpOffset(offset as u32)
                },
                false => {
                    arg_counter += 1;
                    ParamSlot::ArgReg
                }
            }
        }).collect::<Vec<_>>();
        arg_slots
    }
}


