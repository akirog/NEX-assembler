use crate::emulator::cpu::instruction_decoder::RegIoKind;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

pub struct IoManager {
    key_buffer: Receiver<u8>,
    rom: Vec<u8>
}

impl IoManager {
    pub fn new(key_receiver: Receiver<u8>) -> IoManager {
        IoManager {
            key_buffer: key_receiver,
            rom: Vec::new()
        }
    }

    pub fn execute(&mut self, kind: RegIoKind, val1: u32, _val2: u32) -> IoAction {
        match kind {
            RegIoKind::Halt => IoAction::Halt,
            RegIoKind::Time => IoAction::StoreReg(chrono::Local::now().timestamp_millis() as u32),
            RegIoKind::Keyboard => {
                let key = self.key_buffer.try_recv().unwrap_or_else(|_| 0);

                IoAction::StoreReg(key as u32)
            },
            RegIoKind::Rom => {
                let addr = val1 as usize;
                let value: u32 = if addr >= self.rom.len() {
                    0
                } else {
                    (self.rom[addr] as u32) << 0    |
                    (self.rom[addr+1] as u32) << 8  |
                    (self.rom[addr+2] as u32) << 16 |
                    (self.rom[addr+3] as u32) << 24
                };

                IoAction::StoreReg(value)
            },
            _ => IoAction::StoreReg(0)
        }
    }

    pub fn load_rom(&mut self, ram: Vec<u8>) {
        self.rom = ram;
    }
}

pub enum IoAction {
    StoreReg(u32),
    Halt,

}