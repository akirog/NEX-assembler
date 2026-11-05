use crate::compiler::assembly_generation::register_manager::RegisterManager;
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

    pub fn generate(&mut self, mir: Vec<MirStmt>) -> &Vec<String> {
        for stmt in &mir {
            self.generate_stmt(stmt);
        }

        &self.assembly
    }

    fn generate_stmt(&mut self, stmt: &MirStmt) {
        match &stmt.kind {
            MirStmtKind::Instruction(instr) => {
                self.generate_instruction(&instr);
            },
            MirStmtKind::Block(block) => {
                self.generate_block(block);
            }
        }
    }

    fn generate_instruction(&mut self, instruction: &MirInstruction) {
        match &instruction.kind {
            MirInstructionKind::Mov { target, value } => {
                let target = self.reg_manager.get_temp_reg(target);

                self.assembly.push(format!("mov {target}, {value}"));
            },
            MirInstructionKind::Operation { target, val1, val2, op } => {
                let target = self.reg_manager.get_temp_reg(target);

                let left = self.ensure_in_reg(&val1);
                let right = self.ensure_in_reg(&val2);

                self.assembly.push(format!("{} {target}, {left}, {right}", op.assembly_text()));
            },
            MirInstructionKind::Store { addr, value } => {
                let addr = self.ensure_in_reg(&addr);
                let value = self.ensure_in_reg(&value);

                self.assembly.push(format!("store [{addr}], {value}"));
            }
            _ => {
                println!("skipped generation: {:?}", instruction);
            }
        }
    }

    fn generate_block(&mut self, block: &MirBlock) {
        for stmt in &block.statements {
            self.generate_stmt(stmt);
        }
    }

    fn ensure_in_reg(&mut self, value: &MirValue) -> String {
        match value.kind {
            MirValueKind::TempReg(reg) => self.reg_manager.get_temp_reg(&reg).to_string(),
            _ => panic!("ENSURE IN REG ON NON REG {:?}", value),
        }
    }
}