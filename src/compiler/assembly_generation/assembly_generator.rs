use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;

pub struct AssemblyGenerator {
    assembly: Vec<String>,
}

impl AssemblyGenerator {
    pub fn new() -> Self {
        Self {
            assembly: Vec::new(),
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
                match value {
                    Constant::Number(num) => {
                        self.assembly.push(format!("mov {target}, {num}"));
                    },
                    Constant::Label(label) => {
                        self.assembly.push(format!("mov {target}, {label}"));
                    }
                }
            },
            MirInstruction::Operation { target, left, right, op } => {
                self.assembly.push(format!("{} {target}, {left}, {right}", op.assembly_text()));
            },
            MirInstruction::ImmOperation { target, left, right, op } => {
                self.assembly.push(format!("{} {target}, {left}, {right}", op.assembly_text()));
            }
            MirInstruction::Store { target: addr, value, byte } => {
                let byte = if *byte { "b" } else { "" };

                match addr {
                    Address::Absolute(addr) => {
                        self.assembly.push(format!("store{byte} [{addr}], {value}"));
                    },
                    Address::FrameOffset(frame_offset) => {
                        self.assembly.push(format!("store{byte} [bp - {frame_offset}], {value}"));
                    },
                }
            },
            MirInstruction::Load { target, addr} => {
                match addr {
                    Address::Absolute(addr) => {
                        self.assembly.push(format!("load {target}, [{addr}]"));
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
                self.assembly.push(format!("b{} {val1}, {val2}, {dest}", cond.assembly_text()));
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
                let mut reg_counter = 0;

                for arg in args {
                    match arg {
                        CallArg::Register(reg) => {
                            self.assembly.push(format!("mov a{reg_counter}, {reg}"));
                            reg_counter += 1;
                        },
                        _ => {}
                    }

                }

                self.assembly.push(format!("jrl {dest}"));
            },
            MirInstruction::Ret { value } => {
                // SP = BP + 8     ; Top of old stack
                // BP = [BP]       ; Old Bp
                // ra = [SP - 4]   ; Temp for holding return addr
                // jr ra           ; Return
                self.assembly.push(format!("mov v0, {value}"));

                self.assembly.push("addi sp, bp, 8".to_string());
                self.assembly.push("load bp, [bp]".to_string());
                self.assembly.push("load ra, [sp - 4]".to_string());
                self.assembly.push("jr ra".to_string());
            },
            MirInstruction::Jump { label } => {
                self.assembly.push(format!("j {label}"));
            },
            MirInstruction::AsmText(text) => {
                self.assembly.push(format!("{text}"));
            }
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


        for arg in &block.args {
            match arg {
                FuncArg::Register { arg_reg, to } => {
                    self.assembly.push(format!("store [bp - {to}], a{arg_reg}"));
                },
                FuncArg::Address { arg_addr, to } => {
                    self.assembly.push(format!("load at, [sp + 8 + {arg_addr}]"));
                    self.assembly.push(format!("store [bp - {to}], at"));
                }
            }

        }

        for instr in &block.instructions {
            self.generate_instruction(instr);
        }
        let end_idx = self.assembly.len();
        for line in &mut self.assembly[start_idx..end_idx] {
            *line = format!("\t{}", line)
        }
    }
}