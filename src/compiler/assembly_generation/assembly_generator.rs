use crate::compiler::assembly_generation::register_manager::RegisterManager;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;

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

    pub fn generate(&mut self, mir: Vec<MirStmt>, entry_func: &SymbolId) -> &Vec<String> {
        // Init of program
        self.assembly.push("mov bp 0x100000".to_string());
        self.assembly.push("mov sp, bp".to_string());
        self.assembly.push(format!("j function_{}", entry_func));

        for stmt in &mir {
            self.generate_stmt(stmt);
        }

        &self.assembly
    }

    fn generate_stmt(&mut self, stmt: &MirStmt) {
        match &stmt {
            MirStmt::Block(block) => {
                self.generate_block(block);
            }
        }
        self.assembly.push("".to_string());
    }

    fn generate_instruction(&mut self, instruction: &MirInstruction) {
        match &instruction {
            MirInstruction::Mov { target, value } => {
                let target = self.reg_manager.get_temp_reg(target);
                match value {
                    Constant::Number(num) => {
                        self.assembly.push(format!("mov {target}, {num}"));
                    },
                    Constant::Label(label) => {
                        self.assembly.push(format!("mov {target}, {label}"));
                    }
                }
            },
            MirInstruction::Operation { target, val1, val2, op } => {
                let target = self.reg_manager.get_temp_reg(target);

                let left = self.ensure_in_reg(&val1);
                let right = self.ensure_in_reg(&val2);

                self.assembly.push(format!("{} {target}, {left}, {right}", op.assembly_text()));

                self.reg_manager.free_temp_reg(left);
                self.reg_manager.free_temp_reg(right);
            },
            MirInstruction::Store { target: addr, value, byte } => {
                let value = self.ensure_in_reg(&value);
                let byte = if *byte { "b" } else { "" };

                match addr {
                    Address::Absolute(addr_reg) => {
                        let addr = self.reg_manager.get_temp_reg(addr_reg);
                        self.assembly.push(format!("store{byte} [{addr}], {value}"));
                        self.reg_manager.free_temp_reg(addr);
                        self.reg_manager.free_temp_reg(value);
                    },
                    Address::FrameOffset(frame_offset) => {
                        self.assembly.push(format!("store{byte} [bp - {frame_offset}], {value}"));
                        self.reg_manager.free_temp_reg(value)
                    },
                }
            },
            MirInstruction::Load { target, addr} => {
                let target = self.reg_manager.get_temp_reg(target);
                match addr {
                    Address::Absolute(addr_reg) => {
                        let addr = self.reg_manager.get_temp_reg(addr_reg);
                        self.assembly.push(format!("load {target}, [{addr}]"));
                        self.reg_manager.free_temp_reg(addr);
                    },
                    Address::FrameOffset(frame_offset) => {
                        self.assembly.push(format!("load {target} [bp - {frame_offset}]"));
                    },
                }
            }
            MirInstruction::Label { label } => {
                self.assembly.push(format!("{}:", label));
            },
            MirInstruction::Branch { dest, val1, val2, cond } => {
                let val1 = self.ensure_in_reg(&val1);
                let val2 = self.ensure_in_reg(&val2);

                self.assembly.push(format!("b{} {val1}, {val2}, {dest}", cond.assembly_text()));
                self.reg_manager.free_temp_reg(val1);
                self.reg_manager.free_temp_reg(val2);
            },
            MirInstruction::Indented { instructions } => {
                let start_idx = self.assembly.len();
                for instr in instructions {
                    self.generate_instruction(instr);
                }
                let end_idx = self.assembly.len();
                for line in &mut self.assembly[start_idx..end_idx] {
                    *line = format!("\t{}", line)
                }
            },
            MirInstruction::Call { dest, args } => {
                for (i, arg) in args.iter().enumerate() {
                    let arg = self.ensure_in_reg(&arg);
                    self.assembly.push(format!("mov a{i}, {arg}"));
                }

                match dest {
                    MirValue::Const(constant) => {
                        match constant {
                            Constant::Label(label) => {
                                self.assembly.push(format!("jal {label}"));
                            },
                            _ => panic!("Unexpected jump constant {:#?}", constant),
                        }
                    },
                    _ => {
                        let dest = self.ensure_in_reg(dest);
                        self.assembly.push(format!("call {dest}"));
                    }
                }

            },
            MirInstruction::Ret { value } => {
                // SP = BP + 8     ; Top of old stack
                // BP = [BP]       ; Old Bp
                // ra = [SP - 4]   ; Temp for holding return addr
                // jr ra           ; Return
                let reg = self.ensure_in_reg(value);
                self.assembly.push(format!("mov v0, {reg}"));

                self.assembly.push("addi sp, bp, 8".to_string());
                self.assembly.push("load bp, [bp]".to_string());
                self.assembly.push("load ra, [sp - 4]".to_string());
                self.assembly.push("jr ra".to_string());
            },
            MirInstruction::Jump { label } => {
                self.assembly.push(format!("j {label}"));
            },
        }
    }

    fn generate_block(&mut self, block: &MirBlock) {
        self.assembly.push(format!("{}:", block.label));

        // Set up bp and sp
        let start_idx = self.assembly.len();
        self.assembly.push("; Function init".to_string());
        self.assembly.push("store [sp - 4], ra".to_string());
        self.assembly.push("store [sp - 8], bp".to_string());
        self.assembly.push("subi bp, sp, 8".to_string());
        self.assembly.push(format!("subi sp, bp, {}", block.size));
        self.assembly.push("".to_string());

        for (i, arg) in block.args.iter().enumerate() {
            self.assembly.push(format!("store [bp - {arg}], a{i}"));
        }

        for instr in &block.instructions {
            self.generate_instruction(instr);
        }
        let end_idx = self.assembly.len();
        for line in &mut self.assembly[start_idx..end_idx] {
            *line = format!("\t{}", line)
        }
    }

    fn load_addr(&mut self, addr: &Address) -> &'static str {
        match addr {
            Address::Absolute(reg) => {
                let result = self.reg_manager.get_temp_reg(&TempReg::new());
                self.assembly.push(format!("load {}, [{}]", result, self.reg_manager.get_temp_reg(&reg)));
                result
            },
            Address::FrameOffset(offset) => {
                let result = self.reg_manager.get_temp_reg(&TempReg::new());
                self.assembly.push(format!("load {}, [bp - {}]", result, offset));
                result
            },
        }
    }

    fn ensure_in_reg(&mut self, value: &MirValue) -> &'static str {
        match value {
            MirValue::Load(addr) => {
                self.load_addr(&addr)
            },
            MirValue::Const(constant) => {
                let reg = self.reg_manager.get_temp_reg(&TempReg::new());
                match constant {
                    Constant::Number(num) => {
                        self.assembly.push(format!("mov {}, {}", reg, num));
                    },
                    Constant::Label(label) => {
                        self.assembly.push(format!("mov {}, {}", reg, label));
                    }
                }

                reg
            },
            MirValue::Register(reg) => {
                self.reg_manager.get_temp_reg(&reg)
            },
            MirValue::FrameOffsetAddress(offset) => {
                let reg = self.reg_manager.get_temp_reg(&TempReg::new());
                self.assembly.push(format!("subi {reg}, bp, {offset}"));
                reg
            },
            MirValue::Array(values) => {
                panic!("Cannot ensure array in register {:?}", values)
            }
        }
    }
}