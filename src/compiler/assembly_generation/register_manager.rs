#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
    T11,
    T12,
    T13,
    T14,
    T15,
    T16,
    T17,
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Register::T0 => "t0",
            Register::T1 => "t1",
            Register::T2 => "t2",
            Register::T3 => "t3",
            Register::T4 => "t4",
            Register::T5 => "t5",
            Register::T6 => "t6",
            Register::T7 => "t7",
            Register::T8 => "t8",
            Register::T9 => "t9",
            Register::T10 => "t10",
            Register::T11 => "t11",
            Register::T12 => "t12",
            Register::T13 => "t13",
            Register::T14 => "t14",
            Register::T15 => "t15",
            Register::T16 => "t16",
            Register::T17 => "t17",
        };

        write!(f, "{name}")
    }
}

impl Register {
    pub const ALL: [Register; 18] = [
        Register::T0,
        Register::T1,
        Register::T2,
        Register::T3,
        Register::T4,
        Register::T5,
        Register::T6,
        Register::T7,
        Register::T8,
        Register::T9,
        Register::T10,
        Register::T11,
        Register::T12,
        Register::T13,
        Register::T14,
        Register::T15,
        Register::T16,
        Register::T17,
    ];
}



pub struct RegisterManager {
    free_regs: Vec<Register>,
    used_regs: Vec<Register>,
}

impl Default for RegisterManager {
    fn default() -> Self {
        RegisterManager::new()
    }
}

impl RegisterManager {
    pub fn new() -> RegisterManager {
        RegisterManager {
            free_regs: Vec::from(Register::ALL),
            used_regs: Vec::new(),
        }
    }

    pub fn get_temp_reg(&mut self) -> Register {
        let Some(reg) = self.free_regs.pop() else { panic!("No temp registers left") };
        self.used_regs.push(reg);
        reg
    }
    
    pub fn free_temp_reg(&mut self, reg: Register) {
        self.free_regs.push(reg);
        for (i, check_reg) in self.used_regs.iter().enumerate() {
            if check_reg == &reg {
                self.used_regs.remove(i);
                return;
            }
        };
        panic!("Attempt to double free temp reg '{}'", reg);
    }
}