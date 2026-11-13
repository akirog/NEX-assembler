use crate::compiler::assembly_generation::register_manager::RegisterManager;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::operations::{ImmOperation, Operation};
use crate::compiler::ir::thir::*;
use crate::compiler::ir::types::type_table::TypeTable;
use crate::compiler::ir::types::{HirWrittenField, TypeId, TypeKind};
use std::cmp::max;
use std::collections::HashMap;



pub struct ThirLowerer<'a> {
    type_table: &'a TypeTable,
    local_vars: Vec<Vec<SymbolId>>,
    var_offsets: HashMap<SymbolId, u32>,
    frame_size: u32,
    total_frame_size: u32,
    functions: Vec<SymbolId>,
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
            functions: Vec::new(),
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
        let mut reg_counter = 0;
        let mut field_addr_counter = 0;

        for field in fields {
            self.local_vars.last_mut().unwrap().push(field.id.clone());
            let field_type = self.type_table.get_symbol_type_id(&field.id).unwrap();
            let field_size = self.type_table.get_type_size(&field_type) as u32;
            self.frame_size += field_size;
            self.total_frame_size = self.frame_size;
            self.var_offsets.insert(field.id.clone(), self.frame_size);

            let arg = if self.type_table.is_aggregate(&field_type) {
                field_addr_counter += field_size;
                FuncArg::Address {
                    arg_addr: field_addr_counter - field_size,
                    to: field_addr_counter,
                }
            } else {
                reg_counter += 1;
                FuncArg::Register {
                    arg_reg: reg_counter - 1,
                    to: self.frame_size
                }
            };

            args.push(arg);
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
                ThirExprKind::StructLiteral { ty, fields: values } => {
                    let struct_ty = self.type_table.get_type(ty);
                    let fields = match struct_ty {
                        TypeKind::Struct { fields } => fields,
                        _ => unreachable!()
                    };

                    for (i, field) in values.iter().enumerate() {
                        self.lower_expr_into(&*field.value, &curr_addr, &fields.types()[i], instructions);
                        let value_size = self.type_table.get_type_size(&fields.types()[i]);

                        curr_addr.add(value_size as i32, instructions);
                    }
                }
                ThirExprKind::Identifier(id) => {
                    // For aggregate typed variables we just memory copy
                    let mut var_addr = self.generate_expr_address(expr, instructions);

                    let reg = self.reg_manager.get_temp_reg();
                    for _ in 0..self.type_table.get_type_size(&self.type_table.get_symbol_type_id(id).unwrap()) {
                        instructions.push(MirInstruction::Load {
                            target: reg,
                            addr: var_addr.clone(),
                        });

                        instructions.push(MirInstruction::Store {
                            target: curr_addr.clone(),
                            value: reg,
                            byte: true
                        });


                        var_addr.add(1, instructions);
                        curr_addr.add(1, instructions);
                    }
                    self.reg_manager.free_temp_reg(reg);
                    var_addr.free_if_reg(&mut self.reg_manager);
                },
                ThirExprKind::ImplicitCoercion { expr, new_type: _ } => {
                    let ThirExprKind::AddressOf(var) = &expr.kind else { panic!("Unhandled implicit coercion") };

                    let var_type = self.type_table.get_type(&self.type_table.get_symbol_type_id(var).unwrap());
                    let TypeKind::Array { ty: _, len } = var_type else {
                        panic!("Unexpected expression for aggregate type")
                    };

                    // Store var addr at address + 0
                    let address = self.get_addr_of_var(var);
                    match address {
                        Address::FrameOffset(offset) => {
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
                if self.functions.contains(id) {
                    return MirValue::Const(Constant::Label(format!("function_{}", id)))
                }

                if self.type_table.is_aggregate(&self.type_table.get_symbol_type_id(id).unwrap()) {
                    panic!("Aggregate in expression");
                }

                MirValue::Load(self.get_addr_of_var(id))
            },
            ThirExprKind::FunctionCall { function, arguments } => {
                let mut lowered_args = Vec::new();

                for arg in arguments {
                    if self.type_table.is_aggregate(&arg.ty.id) {
                        // Expand stack by arg size
                        let size = self.type_table.get_type_size(&arg.ty.id);
                        let addr = Address::FrameOffset(self.frame_size as usize);
                        self.frame_size += size as u32;
                        self.total_frame_size = max(self.frame_size, self.total_frame_size);


                        // Insert arg at addr
                        self.lower_expr_into(arg, &addr, &arg.ty.id, instructions);


                        lowered_args.push(CallArg::Address(addr));

                    } else {
                        lowered_args.push(
                            CallArg::Register(
                                self.lower_expr(arg, instructions).ensure_in_reg(instructions, &mut self.reg_manager)
                            )
                        );
                    }
                }

                let mut function = self.lower_expr(function, instructions);

                let instr = MirInstruction::Call {
                    dest: function.ensure_in_reg(instructions, &mut self.reg_manager),
                    args: lowered_args.clone()
                };
                instructions.push(instr);
                function.free_if_reg(&mut self.reg_manager);

                for reg in lowered_args {
                    if let CallArg::Register(reg) = reg {
                        self.reg_manager.free_temp_reg(reg);
                    }
                };

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
                let addr = match self.get_addr_of_var(var) {
                    Address::FrameOffset(offset) => offset,
                    _ => panic!("Idk what to do here")
                };

                MirValue::FrameOffsetAddress(addr)
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
            ThirExprKind::MemberAccess { left, member } => {
                let mut left_addr = self.generate_expr_address(&left, instructions);

                let Some(addr_add) = self.type_table.get_member_offset(&left.ty.id, member) else {
                    panic!("Could not get member offset");
                };

                left_addr.add(addr_add as i32, instructions);
                let reg = self.reg_manager.get_temp_reg();

                instructions.push(MirInstruction::Load {
                    target: reg,
                    addr: left_addr.clone()
                });

                left_addr.free_if_reg(&mut self.reg_manager);

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
                    Address::FrameOffset(offset) => MirValue::FrameOffsetAddress(offset),
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
            }
            _ => panic!("not implemented {}", expr.pretty_print(&self.type_table)),
        }
    }

    fn get_temp_label(&mut self) -> String {
        self.label_count += 1;
        format!("temp_label_{}", self.label_count - 1)
    }

    fn get_addr_of_var(&self, symbol: &SymbolId) -> Address {
        Address::FrameOffset(self.var_offsets[symbol] as usize)
    }
}
