use std::time::Instant;
use crate::emulator::instruction_decoder::RegIoKind;

pub struct IoManager {

}

impl IoManager {
    pub fn new() -> IoManager {
        IoManager {

        }
    }

    pub fn execute(&mut self, kind: RegIoKind, val1: u32, val2: u32) -> IoAction {

        match kind {
            RegIoKind::Halt => IoAction::Halt,
            RegIoKind::Time => IoAction::StoreReg(chrono::Local::now().timestamp_millis() as u32),
            _ => IoAction::StoreReg(0)
        }
    }
}

pub enum IoAction {
    StoreReg(u32),
    Halt,

}