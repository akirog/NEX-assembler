use std::collections::HashMap;
use crate::compiler::ir::mir::TempReg;

static REGISTERS: [&'static str; 18]  = [
    "t0",
    "t1",
    "t2",
    "t3",
    "t4",
    "t5",
    "t6",
    "t7",
    "t8",
    "t9",
    "t10",
    "t11",
    "t12",
    "t13",
    "t14",
    "t15",
    "t16",
    "t17"
];

pub struct RegisterManager {
    free_regs: Vec<&'static str>,
    used_regs: Vec<&'static str>,

    temp_regs: HashMap<TempReg, &'static str>,
}

impl RegisterManager {
    pub fn new() -> RegisterManager {
        RegisterManager {
            free_regs: Vec::from(REGISTERS),
            used_regs: Vec::new(),

            temp_regs: HashMap::new(),
        }
    }

    pub fn get_temp_reg(&mut self, temp: &TempReg) -> &'static str {
        if self.temp_regs.contains_key(temp) {
            self.temp_regs[temp]
        } else {
            let reg = self.free_regs.pop().unwrap();
            self.used_regs.push(reg);
            self.temp_regs.insert(temp.clone(), reg);
            reg
        }
    }
    
    pub fn free_temp_reg(&mut self, reg: &'static str) {
        self.free_regs.push(reg);
        for (i, check_reg) in self.used_regs.iter().enumerate() {
            if check_reg == &reg {
                self.used_regs.remove(i);
                return;
            }
        };
        panic!("Undefined temp reg '{}'", reg);
    }
}