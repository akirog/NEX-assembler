use crate::emulator::emulator_controller::{GpuMemUpdate, SCREEN_AREA, VIDEO_MEMORY_SIZE, VIDEO_MEMORY_START};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::mpsc::Sender;

pub struct Gpu {
    pub mem_update_receiver: Receiver<GpuMemUpdate>,
    pub screen_data_sender: Sender<Box<[u8]>>,
    pub instr_memory: [u8; 16],
    pub vram: Box<[u8; VIDEO_MEMORY_SIZE as usize]>,
}

impl Gpu {
    pub fn new(mem_update_receiver: Receiver<GpuMemUpdate>, screen_data_sender: Sender<Box<[u8]>>) -> Self {
        Self {
            mem_update_receiver,
            screen_data_sender,
            instr_memory: [0; 16],
            vram: Box::new([0; VIDEO_MEMORY_SIZE as usize]),
        }
    }

    pub fn start(&mut self) {
        loop {
            let mut updated = false;
            while let Ok(msg) = self.mem_update_receiver.try_recv() {
                // Update memory
                let address = msg.address - VIDEO_MEMORY_START;

                self.vram[(address + 0) as usize] = (msg.value >> 0) as u8;
                self.vram[(address + 1) as usize] = (msg.value >> 8) as u8;
                self.vram[(address + 2) as usize] = (msg.value >> 16) as u8;
                self.vram[(address + 3) as usize] = (msg.value >> 24) as u8;

                updated = true;
            }

            if updated {
                // Send updated memory
                let pixel_data: Box<[u8]> = self.vram[0..SCREEN_AREA as usize * 4].into();
                self.screen_data_sender.send(
                    pixel_data,
                ).expect("Couldn't send screen data");
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}