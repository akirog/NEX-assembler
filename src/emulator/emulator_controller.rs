use std::sync::{Arc, Mutex};
use winit::event_loop::{ControlFlow, EventLoop};
use crate::emulator::cpu::cpu::{EmuState, Cpu, REGISTER_COUNT};
use crate::emulator::cpu::gpu::Gpu;
use crate::emulator::cpu::screen::{Screen, ScreenSetting};
use crate::emulator::ui::tui::{Tui, TuiAction};



/// The amount of bytes for each pixel in the screen
pub const SCREEN_BPP: u32 = 4;


pub const SCREEN_RES: u32 = 32;
pub const SCREEN_WIDTH: u32 = SCREEN_RES * 4;
pub const SCREEN_HEIGHT: u32 = SCREEN_RES * 3;
pub const SCREEN_AREA: u32 = SCREEN_WIDTH * SCREEN_HEIGHT;

pub const VIDEO_MEMORY_START: u32 = 0x08000000;

pub const VIDEO_MEMORY_END: u32 = 0x08100000;
pub const VIDEO_MEMORY_SIZE: u32 = VIDEO_MEMORY_END - VIDEO_MEMORY_START;


pub const ASCII_8_SCREEN_WIDTH: u32 = 96;
pub const ASCII_8_SCREEN_HEIGHT: u32 = 40;


pub struct EmulatorController {}

impl EmulatorController {
    pub fn new() -> EmulatorController {
        EmulatorController {

        }
    }

    pub fn run(&mut self, ram_init: Option<String>, rom_init: Option<String>) -> Result<(), String> {
        assert!(VIDEO_MEMORY_SIZE > SCREEN_AREA * SCREEN_BPP, "Not enough video memory for screen area");

        let (cpu_info_sender, cpu_info_receiver) =
            std::sync::mpsc::channel::<CpuTuiData>();

        let (tui_action_sender, tui_action_receiver) =
            std::sync::mpsc::channel::<TuiAction>();

        let (gpu_memory_update_sender, gpu_memory_update_receiver) =
            std::sync::mpsc::channel::<GpuMemUpdate>();

        let (screen_data_sender, screen_data_receiver) =
            std::sync::mpsc::channel::<Box<[u8]>>();

        let (key_sender, key_receiver) = std::sync::mpsc::channel::<u8>();

        let (screen_setting_sender, screen_setting_receiver) =
            std::sync::mpsc::channel::<ScreenSetting>();


        let shutdown_requested = Arc::new(Mutex::new(false));

        let mut tui = Tui::new(cpu_info_receiver, tui_action_sender, shutdown_requested.clone());

        let mut cpu = Cpu::new(
            cpu_info_sender,
            tui_action_receiver,
            shutdown_requested.clone(),
            key_receiver,
            screen_setting_sender
        );
        if let Some(rom_path) = rom_init {
            cpu.load_init_rom(rom_path);
        }

        if let Some(ram_path) = ram_init {
            cpu.load_init_ram(ram_path);
        }

        let mut gpu = Gpu::new(
            gpu_memory_update_receiver,
            screen_data_sender,
            shutdown_requested.clone()
        );

        let mut screen = Screen::new(
            screen_data_receiver,
            shutdown_requested,
            key_sender,
            screen_setting_receiver
        );

        let emu_handle = std::thread::spawn(move || {
            cpu.start(gpu_memory_update_sender);
        });

        let tui_handle = std::thread::spawn(move || {
            tui.start();
        });

        let gpu_handle = std::thread::spawn(move || {
            gpu.start();
        });

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut screen).expect("TODO: panic message");
        
        emu_handle.join().unwrap();
        tui_handle.join().unwrap();
        gpu_handle.join().unwrap();

        ratatui::restore();

        Ok(())
    }
}


/// Data sent from the emulator to the tui
#[derive(Default)]
pub struct CpuTuiData {
    pub instruction_pointer: u32,
    pub registers: [u32; REGISTER_COUNT],
    pub memory_slice: Vec<u8>,
    pub state: EmuState,
    pub instr_per_sec: u32,
}

#[derive(Default)]
pub struct GpuMemUpdate {
    pub address: u32,
    pub value: u32,
    pub byte: bool,
}