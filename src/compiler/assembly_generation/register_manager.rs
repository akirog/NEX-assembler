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
}

impl RegisterManager {
    pub fn new() -> RegisterManager {
        RegisterManager {
            free_regs: Vec::from(REGISTERS),
            used_regs: Vec::new(),
        }
    }

    pub fn get_temp_reg(&mut self) -> &'static str {
        let reg = self.free_regs.pop().unwrap();
        self.used_regs.push(reg);
        reg
    }
    
    pub fn free_temp_reg(&mut self, reg: &'static str) {
        self.free_regs.push(reg);
        self.used_regs.remove(self.used_regs.binary_search(&reg).unwrap());
    }
}