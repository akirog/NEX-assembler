use crate::emulator::emulator_controller::{GpuMemUpdate, VIDEO_MEMORY_SIZE, VIDEO_MEMORY_START};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use crate::emulator::cpu::screen::ScreenData;

pub struct Gpu {
    pub mem_update_receiver: Receiver<GpuMemUpdate>,
    pub screen_data_sender: Sender<ScreenData>,
    pub _instr_memory: [u8; 16],
    pub vram: Box<[u8; VIDEO_MEMORY_SIZE as usize]>,
    pub shutdown_requested: Arc<Mutex<bool>>
}

impl Gpu {
    pub fn new(mem_update_receiver: Receiver<GpuMemUpdate>, screen_data_sender: Sender<ScreenData>, shutdown_requested: Arc<Mutex<bool>>) -> Self {
        Self {
            mem_update_receiver,
            screen_data_sender,
            _instr_memory: [0; 16],
            vram: Box::new([0; VIDEO_MEMORY_SIZE as usize]),
            shutdown_requested
        }
    }

    pub fn start(&mut self) {
        loop {
            let mut updated = false;
            while let Ok(msg) = self.mem_update_receiver.try_recv() {
                // Update memory
                let address = msg.address - VIDEO_MEMORY_START;

                self.vram[(address + 0) as usize] = (msg.value >> 0) as u8;
                if !msg.byte {
                    self.vram[(address + 1) as usize] = (msg.value >> 8) as u8;
                    self.vram[(address + 2) as usize] = (msg.value >> 16) as u8;
                    self.vram[(address + 3) as usize] = (msg.value >> 24) as u8;
                }

                updated = true;
            }

            if updated {
                // Send updated memory
                let pixel_data: Box<[u8]> = self.vram[0..VIDEO_MEMORY_SIZE as usize].into();
                self.screen_data_sender.send(
                    ScreenData::Data(pixel_data),
                ).expect("Couldn't send screen data");
            }

            if self.shutdown_requested.lock().unwrap().clone() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}