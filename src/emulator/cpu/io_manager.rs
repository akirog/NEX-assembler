use std::time::Duration;
use crate::emulator::cpu::instruction_decoder::RegIoKind;

pub struct IoManager {}

impl IoManager {
    pub fn new() -> IoManager {
        IoManager {

        }
    }

    pub fn execute(&mut self, kind: RegIoKind, val1: u32, val2: u32) -> IoAction {

        match kind {
            RegIoKind::Halt => IoAction::Halt,
            RegIoKind::Time => IoAction::StoreReg(chrono::Local::now().timestamp_millis() as u32),
            RegIoKind::Keyboard => {
                let key = crossterm::event::read().unwrap();
                if let Some(key) = key.as_key_event() {
                    IoAction::StoreReg(key.code.as_char().unwrap() as u32)
                } else {
                    IoAction::StoreReg(0)
                }
            }
            _ => IoAction::StoreReg(0)
        }
    }
}

pub enum IoAction {
    StoreReg(u32),
    Halt,

}