use crate::emulator::emulator_controller::{ASCII_8_SCREEN_HEIGHT, ASCII_8_SCREEN_WIDTH, SCREEN_RES};
use pixels::{Pixels, ScalingMode, SurfaceTexture};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Screen {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    filled_frame_receiver: Receiver<Box<[u8]>>,
    shutdown_requested: Arc<Mutex<bool>>,
    key_sender: Sender<u8>,

    screen_data: Vec<u8>,
    current_mode: ScreenMode,
}


impl Screen {
    pub fn new(filled_frame_receiver: Receiver<Box<[u8]>>, shutdown_requested: Arc<Mutex<bool>>, key_sender: Sender<u8>) -> Self {
        Screen {
            window: None,
            pixels: None,
            filled_frame_receiver,
            shutdown_requested,
            key_sender,

            screen_data: vec![],
            current_mode: ScreenMode::Ascii8 { foreground_color: 0xFFFFFFFF, background_color: 0xFF000000 },
        }
    }
}

impl ApplicationHandler for Screen {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }

        let window_attributes = WindowAttributes::default();
        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let physical_window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            physical_window_size.width,
            physical_window_size.height,
            window.clone(),
        );

        let mut pixels = Pixels::new(ASCII_8_SCREEN_WIDTH*8, ASCII_8_SCREEN_HEIGHT*8, surface_texture).unwrap();
        pixels.set_scaling_mode(ScalingMode::PixelPerfect);
        pixels.render().unwrap();
        self.pixels = Some(pixels);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                *self.shutdown_requested.lock().as_deref_mut().unwrap() = true;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let Some(window) = &self.window else { return; };

                let mut newest_frame = None;
                while let Ok(new_pixels) = self.filled_frame_receiver.try_recv() {
                    newest_frame = Some(new_pixels);
                }

                if let Some(new_data) = newest_frame {
                    self.screen_data = new_data.to_vec();
                }

                let Some(pixels) = self.pixels.as_mut() else { return; };
                match self.current_mode {
                    ScreenMode::Pixel32 => {
                        pixels.frame_mut().copy_from_slice(self.screen_data.as_slice());
                    }
                    ScreenMode::Ascii8 { foreground_color, background_color } => {
                        let frame = pixels.frame_mut();

                        render_ascii8(frame, foreground_color, background_color, self.screen_data.as_slice());
                    }
                    ScreenMode::Ascii32 => {

                    }
                }

                pixels.render().unwrap();

                if self.shutdown_requested.lock().unwrap().clone() {
                    event_loop.exit();
                }

                std::thread::sleep(Duration::from_millis(33));
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if event.state == winit::event::ElementState::Released { return; }
                if let Some(text) = event.text {
                    for c in text.as_bytes() {
                        self.key_sender.send(*c).expect("Couldnt send character");
                    }
                }

            }
            WindowEvent::Resized(new_size) => {
                let Some(window) = &self.window else { return; };
                let Some(pixels) = self.pixels.as_mut() else { return; };

                pixels
                    .resize_surface(new_size.width, new_size.height)
                    .unwrap();

                window.request_redraw();
            }
            _ => {}
        }
    }
}


fn render_ascii8(frame: &mut [u8], foreground_color: u32, background_color: u32, data: &[u8]) {
    let mut curr_x = 0;
    let mut curr_y = 0;

    for d in data {
        let c = *d as char;
        if !c.is_ascii() {
            curr_x += 8;
            if curr_x + 8 > ASCII_8_SCREEN_WIDTH as usize * 8 {
                curr_x = 0;
                curr_y += 8;
            }
            continue;
        }

        let bitmap = font8x8::BASIC_UNICODE[c as usize];

        for (y, row) in bitmap.byte_array().iter().enumerate() {
            for x in 0..8 {
                let active = row >> x & 1 == 1;

                let color = if active { foreground_color } else { background_color };

                let index =
                    (
                        (curr_x + x) +
                        (curr_y + y) * ASCII_8_SCREEN_WIDTH as usize * 8
                    ) * 4;

                if index + 4 > frame.len() {
                    return;
                }

                frame[index + 0] = (color >> 0) as u8;
                frame[index + 1] = (color >> 8) as u8;
                frame[index + 2] = (color >> 16) as u8;
                frame[index + 3] = (color >> 24) as u8;
            }
        }

        curr_x += 8;
        if curr_x + 8 > ASCII_8_SCREEN_WIDTH as usize * 8 {
            curr_x = 0;
            curr_y += 8;
        }
    }
}


enum ScreenMode {
    Ascii8 {
        foreground_color: u32,
        background_color: u32,
    },
    Ascii32,
    Pixel32,
}