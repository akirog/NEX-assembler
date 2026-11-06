use std::fmt::format;
use crate::compiler::assembly_generation::register_manager::RegisterManager;
use crate::compiler::ir::hir::SymbolId;
use crate::compiler::ir::mir::*;
use crate::compiler::ir::operations::*;

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

                self.assembly.push(format!("mov {target}, {value}"));
            },
            MirInstruction::Operation { target, val1, val2, op } => {
                let target = self.reg_manager.get_temp_reg(target);

                let left = self.ensure_in_reg(&val1);
                let right = self.ensure_in_reg(&val2);

                self.assembly.push(format!("{} {target}, {left}, {right}", op.assembly_text()));
            },
            MirInstruction::Store { target: addr, value } => {
                let value = self.ensure_in_reg(&value);
                match addr {
                    Address::Absolute(addr_reg) => {
                        self.assembly.push(format!("store [{}], {value}", self.reg_manager.get_temp_reg(addr_reg)));
                    },
                    Address::FrameOffset(frame_offset) => {
                        self.assembly.push(format!("store [bp - {frame_offset}], {value}"));
                    }
                }
            },
            MirInstruction::Label { label } => {
                self.assembly.push(format!("{}:", label));
            },
            MirInstruction::Branch { dest, val1, val2, cond } => {
                let val1 = self.ensure_in_reg(&val1);
                let val2 = self.ensure_in_reg(&val2);

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
            }
            _ => {
                println!("skipped generation: {:?}", instruction);
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

        for instr in &block.instructions {
            self.generate_instruction(instr);
        }
        let end_idx = self.assembly.len();
        for line in &mut self.assembly[start_idx..end_idx] {
            *line = format!("\t{}", line)
        }
    }

    fn load_addr(&mut self, addr: &Address) -> String {
        match addr {
            Address::Absolute(reg) => {
                let result = self.reg_manager.get_temp_reg(&TempReg::new());
                self.assembly.push(format!("load {}, [{}]", result, self.reg_manager.get_temp_reg(&reg)));
                result.to_string()
            },
            Address::FrameOffset(offset) => {
                let result = self.reg_manager.get_temp_reg(&TempReg::new());
                self.assembly.push(format!("load {}, [bp - {}]", result, offset));
                result.to_string()
            }
        }
    }

    fn ensure_in_reg(&mut self, value: &MirValue) -> String {
        match value {
            MirValue::Load(addr) => {
                self.load_addr(&addr)
            },
            MirValue::Number(number) => {
                let reg = self.reg_manager.get_temp_reg(&TempReg::new());
                self.assembly.push(format!("mov {}, {}", reg, number));
                reg.to_string()
            },
            MirValue::Register(reg) => {
                self.reg_manager.get_temp_reg(&TempReg::new()).to_string()
            }
        }
    }
}