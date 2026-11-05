use crate::emulator::cpu::emu::{EmuState, Emulator, REGISTER_COUNT};
use crate::emulator::ui::tui::{Tui, TuiAction};

pub struct EmulatorController {}

impl EmulatorController {
    pub fn new() -> EmulatorController {
        EmulatorController {

        }
    }

    pub fn run(&mut self, ram_init: Option<String>, rom_init: Option<String>) -> Result<(), String> {
        let mut tui = Tui::new();

        let mut emulator = Emulator::new();
        if let Some(rom_path) = rom_init {
            emulator.load_init_rom(rom_path);
        }

        if let Some(ram_path) = ram_init {
            emulator.load_init_ram(ram_path);
        }

        let (info_sender, info_receiver) = std::sync::mpsc::channel::<EmuTuiData>();
        let (action_sender, action_receiver) = std::sync::mpsc::channel::<TuiAction>();


        let emu_handle = std::thread::spawn(move || {
            emulator.start(info_sender, action_receiver);
        });

        let tui_handle = std::thread::spawn(move || {
            tui.start(info_receiver, action_sender);
        });

        emu_handle.join().unwrap();
        tui_handle.join().unwrap();

        ratatui::restore();

        Ok(())
    }
}


/// Data sent from the emulator to the tui
#[derive(Default)]
pub struct EmuTuiData {
    pub instruction_pointer: u32,
    pub registers: [u32; REGISTER_COUNT],
    pub memory_slice: Vec<u8>,
    pub state: EmuState,
    pub instr_per_sec: u32,
}