use crate::emulator::emulator_controller::EmulatorController;

pub mod emulator_controller;
pub mod ui;
pub mod cpu;

pub fn run_emulator(ram_init: Option<String>, rom_init: Option<String>) -> Result<(), String> {
    let mut controller = EmulatorController::new();
    controller.run(ram_init, rom_init)
}