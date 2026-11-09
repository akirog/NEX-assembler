use crate::emulator::cpu::instruction_decoder::{CpuInstructionKind, ImmArithKind, ImmBranchKind, ImmOp, InstructionDecoder, JmpKind, RegArithKind, RegJumpKind, RegOp};
use crate::emulator::cpu::io_manager::{IoAction, IoManager};
use crate::emulator::cpu::mem_manager::MemoryManager;
use crate::emulator::ui::tui::{MemViewMode, TuiAction};
use crate::emulator::emulator_controller::EmuTuiData;
use std::cmp::max;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub const REGISTER_COUNT: usize = 32;


pub struct Emulator {
    instruction_decoder: InstructionDecoder,
    pub memory_manager: MemoryManager,
    io_manager: IoManager,
    registers: [u32; REGISTER_COUNT],
    pub instruction_pointer: u32,
    state: EmuState,
}



impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            instruction_decoder: InstructionDecoder::new(),
            io_manager: IoManager::new(),
            memory_manager: MemoryManager::new(),
            registers: [0; REGISTER_COUNT],
            instruction_pointer: 0,
            state: EmuState::Paused
        }
    }

    pub fn start(&mut self, info_sender: Sender<EmuTuiData>, action_receiver: Receiver<TuiAction>) {
        let mut last_data_send = Instant::now();
        let mut run_time = Duration::new(0, 0);
        let mut instructions = 0;

        let mut requested_mem_area = (0, 400);

        loop {
            while let Ok(action) = action_receiver.try_recv() {
                match action {
                    TuiAction::Quit => return,
                    TuiAction::Step(count) => {
                        for _ in 0..count {
                            if self.execute_instruction() { break };
                        }
                    },
                    TuiAction::Run => {
                        run_time = Duration::new(0, 0);
                        instructions = 0;
                        self.state = EmuState::Running
                    },
                    TuiAction::Stop => {
                        self.state = EmuState::Paused
                    },
                    TuiAction::UpdateMemArea(area) => {
                        requested_mem_area = area;
                    },
                    TuiAction::SetMemViewMode(_) => {}
                }
            }

            match self.state {
                EmuState::Paused => { std::thread::sleep(Duration::from_millis(20)); },
                EmuState::Running => {
                    let start_time = Instant::now();
                    const COUNT: usize = 500_000;
                    for _ in 0..COUNT {
                        if self.execute_instruction() { break };
                    }
                    instructions += COUNT;
                    run_time += start_time.elapsed();
                }
            }

            if last_data_send.elapsed() > Duration::from_millis(33) {
                if self.memory_manager.ram().len() < (requested_mem_area.0 + requested_mem_area.1) as usize {
                    self.memory_manager.store_8(requested_mem_area.0 + requested_mem_area.1, 0);
                }

                info_sender.send(EmuTuiData {
                    instruction_pointer: self.instruction_pointer,
                    registers: self.registers,
                    memory_slice: self.memory_manager.ram().split_at(requested_mem_area.0 as usize).1.split_at(requested_mem_area.1 as usize).0.to_vec(),
                    state: self.state.clone(),
                    instr_per_sec: (instructions as u128 / max(run_time.as_millis() / 1000, 1)) as u32,
                }).unwrap();
                last_data_send = Instant::now();
            }
        }
    }

    fn execute_instruction(&mut self) -> bool {
        let instruction = self.instruction_decoder.get_instruction(&self);
        let mut did_jump = false;

        let result = match instruction.kind {
            CpuInstructionKind::Imm { op, dst, src1, imm } => {
                self.run_imm_instr(op, dst, src1, imm, &mut did_jump);
                false
            },
            CpuInstructionKind::Reg { op, src1, src2, dst, flags } => {
                self.run_reg_instr(op, src1, src2, dst, flags, &mut did_jump)
            },
            CpuInstructionKind::Jmp { kind, addr } => {
                self.run_jmp_instr(kind, addr, &mut did_jump);
                false
            }
        };

        if !did_jump {
            self.instruction_pointer = self.instruction_pointer.wrapping_add(4);
        }

        result
    }

    fn run_imm_instr(&mut self, op: ImmOp, dst: u8, src1: u8, imm: u16, did_jump: &mut bool) {
        match op {
            ImmOp::Arith(kind) => {
                let imm = imm as i16 as i32;
                let val1 = self.get_reg(src1) as i32;

                let result = match kind {
                    ImmArithKind::ADD => val1.wrapping_add(imm),
                    ImmArithKind::ADDHI => val1.wrapping_add(imm<<16),
                    ImmArithKind::SUB => val1.wrapping_sub(imm),
                    ImmArithKind::MUL => val1.wrapping_mul(imm),
                    ImmArithKind::DIV => val1.wrapping_div(imm),
                    ImmArithKind::OR => val1 | imm,
                    ImmArithKind::SHL => val1.wrapping_shl(imm as u32),
                    ImmArithKind::SHR => val1.wrapping_shr(imm as u32),
                } as u32;

                self.set_reg(dst, result);
            },
            ImmOp::Branch(kind) => {
                let val1 = self.get_reg(dst);
                let val2 = self.get_reg(src1);

                let jump: bool = match kind {
                    ImmBranchKind::Unconditional => true,
                    ImmBranchKind::Greater => (val1 as i32) > (val2 as i32),
                    ImmBranchKind::Less => (val1 as i32) < (val2 as i32),
                    ImmBranchKind::Equal => val1 == val2,
                    ImmBranchKind::NotEqual => val1 != val2,
                    ImmBranchKind::Above => val1 > val2,
                    ImmBranchKind::Below => val1 < val2,
                };

                if jump {
                    let jmp_addr = self.instruction_pointer as i32 + ((imm as i16 as i32) << 2);
                    self.instruction_pointer = jmp_addr as u32;
                    *did_jump = true;
                }
            },
            ImmOp::Mem(mem_fields) => {
                let addr = ((self.get_reg(src1) as i32) + (imm as i16 as i32)) as u32;

                if mem_fields.store {
                    let val = self.get_reg(dst);

                    if mem_fields.byte {
                        self.memory_manager.store_8(addr, (val&0xFF) as u8);
                    } else {
                        self.memory_manager.store_32(addr, val);
                    }
                } else {
                    let val = if mem_fields.byte {
                        self.memory_manager.load_8(addr) as u32
                    } else {
                        self.memory_manager.load_32(addr)
                    };

                    self.set_reg(dst, val);
                }
            }
        }
    }

    fn run_reg_instr(&mut self, op: RegOp, src1: u8, src2: u8, dst: u8, _flags: u16, did_jump: &mut bool) -> bool {
        match op {
            RegOp::Arith(kind) => {
                let val1 = self.get_reg(src1);
                let val2 = self.get_reg(src2);

                let result = match kind {
                    RegArithKind::ADD => (val1 as i32).wrapping_add(val2 as i32) as u32,
                    RegArithKind::SUB => (val1 as i32).wrapping_sub(val2 as i32) as u32,
                    RegArithKind::MUL => (val1 as i32).wrapping_mul(val2 as i32) as u32,
                    RegArithKind::DIV => (val1 as i32).wrapping_div(val2 as i32) as u32,
                    RegArithKind::AND => val1 & val2,
                    RegArithKind::OR => val1 | val2,
                    RegArithKind::XOR => val1 ^ val2,
                    RegArithKind::SHL => val1 << val2,
                    RegArithKind::SHR => val1 >> val2,
                    RegArithKind::EQ => (val1 == val2) as u32,
                    RegArithKind::NE => (val1 != val2) as u32,
                    RegArithKind::LT => (val1 < val2) as u32,
                    RegArithKind::LTE => (val1 <= val2) as u32,
                    RegArithKind::NEG => -(val1 as i32) as u32,
                    RegArithKind::MOD => val1 % val2,
                };

                self.set_reg(dst, result);
            },
            RegOp::Jump(kind) => {
                let dst = self.get_reg(src1);

                match kind {
                    RegJumpKind::Jump => {},
                    RegJumpKind::JumpLink => { self.set_reg(31, self.instruction_pointer+4) }
                };

                self.instruction_pointer = dst;
                *did_jump = true;
            },
            RegOp::Io(kind) => {
                match self.io_manager.execute(kind, self.get_reg(src1), self.get_reg(src2)) {
                    IoAction::Halt => {
                        self.state = EmuState::Paused;
                        return true;
                    },
                    IoAction::StoreReg(val) => { self.set_reg(dst, val); },
                }
            },
            RegOp::Int => {

            }
        };

        false
    }

    fn run_jmp_instr(&mut self, kind: JmpKind, addr: u32, did_jump: &mut bool) {
        match kind {
            JmpKind::Jump => {},
            JmpKind::JumpLink => {
                self.set_reg(31, self.instruction_pointer + 4);
            }
        }

        let jmp_addr = self.instruction_pointer as i32 + ((addr as i32) << 2);
        self.instruction_pointer = jmp_addr as u32;
        *did_jump = true;
    }


    pub fn load_init_ram(&mut self, ram: String) {
        let data = std::fs::read(ram).unwrap();

        self.memory_manager.load_init_buff(data);
    }

    pub fn load_init_rom(&mut self, rom_path: String) {
        let data = std::fs::read(rom_path).unwrap();

        self.io_manager.load_rom(data);
    }

    fn get_reg(&self, reg: u8) -> u32 {
        match reg {
            0 => 0,
            _ => self.registers[reg as usize],
        }
    }

    fn set_reg(&mut self, reg: u8, val: u32) {
        match reg {
            0 => {},
            _ => self.registers[reg as usize] = val,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub enum EmuState {
    #[default]
    Paused,
    Running,
}