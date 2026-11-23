use crate::emulator::cpu::instruction_decoder::RegIoKind;
use std::sync::mpsc::{Receiver, Sender};
use crate::emulator::cpu::screen::ScreenSetting;

pub struct IoManager {
    key_buffer: Receiver<u8>,
    rom: Vec<u8>,
    screen_setting_sender: Sender<ScreenSetting>
}

impl IoManager {
    pub fn new(key_receiver: Receiver<u8>, screen_setting_sender: Sender<ScreenSetting>) -> IoManager {
        IoManager {
            key_buffer: key_receiver,
            rom: Vec::new(),
            screen_setting_sender
        }
    }

    pub fn execute(&mut self, kind: RegIoKind, val1: u32, val2: u32) -> IoAction {
        match kind {
            RegIoKind::Halt => IoAction::Halt,
            RegIoKind::Time => IoAction::StoreReg(chrono::Local::now().timestamp_millis() as u32),
            RegIoKind::Keyboard => {
                let key = self.key_buffer.try_recv().unwrap_or_else(|_| 0);

                IoAction::StoreReg(key as u32)
            },
            RegIoKind::Rom => {
                let addr = val1 as usize;
                let value: u32 = if addr+3 >= self.rom.len() {
                    0
                } else {
                    (self.rom[addr] as u32) << 0    |
                    (self.rom[addr+1] as u32) << 8  |
                    (self.rom[addr+2] as u32) << 16 |
                    (self.rom[addr+3] as u32) << 24
                };

                IoAction::StoreReg(value)
            },
            RegIoKind::Screen => {
                let setting = ScreenSetting {
                    option: val1 as u8,
                    value: val2
                };

                self.screen_setting_sender.send(setting).unwrap();

                IoAction::StoreReg(0)
            }
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