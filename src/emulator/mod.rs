use std::path::{Path, PathBuf};
use crate::emulator::emu::Emulator;
use crate::emulator::emulator_controller::EmulatorController;

pub mod emu;
mod mem_manager;
pub mod instruction_decoder;
pub mod tui;
pub mod emulator_controller;

pub fn run_emulator(ram_init: Option<String>, rom_init: Option<String>) -> Result<(), String> {
    let mut controller = EmulatorController::new();
    controller.run(ram_init, rom_init)
}