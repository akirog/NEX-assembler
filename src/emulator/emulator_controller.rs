use winit::event_loop::{ControlFlow, EventLoop};
use crate::emulator::cpu::cpu::{EmuState, Cpu, REGISTER_COUNT};
use crate::emulator::cpu::gpu::Gpu;
use crate::emulator::cpu::screen::Screen;
use crate::emulator::ui::tui::{Tui, TuiAction};

pub const SCREEN_RES: u32 = 32;
pub const SCREEN_AREA: u32 = SCREEN_RES * 4 * SCREEN_RES * 3;

pub const VIDEO_MEMORY_START: u32 = 0x08000000;

pub const VIDEO_MEMORY_END: u32 = 0x08100000;
pub const VIDEO_MEMORY_SIZE: u32 = VIDEO_MEMORY_END - VIDEO_MEMORY_START;

pub struct EmulatorController {}

impl EmulatorController {
    pub fn new() -> EmulatorController {
        EmulatorController {

        }
    }

    pub fn run(&mut self, ram_init: Option<String>, rom_init: Option<String>) -> Result<(), String> {
        let (cpu_info_sender, cpu_info_receiver) =
            std::sync::mpsc::channel::<CpuTuiData>();

        let (tui_action_sender, tui_action_receiver) =
            std::sync::mpsc::channel::<TuiAction>();

        let (gpu_memory_update_sender, gpu_memory_update_receiver) =
            std::sync::mpsc::channel::<GpuMemUpdate>();

        let (screen_data_sender, screen_data_receiver) =
            std::sync::mpsc::channel::<Box<[u8]>>();


        let shutdown_requested = std::sync::Arc::new(std::sync::Mutex::new(false));

        let mut tui = Tui::new(cpu_info_receiver, tui_action_sender);

        let mut cpu = Cpu::new(cpu_info_sender, tui_action_receiver);
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

        let mut screen = Screen {
            window: None,
            pixels: None,
            filled_frame_receiver: screen_data_receiver,
            shutdown_requested
        };

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
}