use crate::emulator::emulator_controller::{ASCII_SCREEN_HEIGHT, ASCII_SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels::{Pixels, ScalingMode, SurfaceTexture};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Screen {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    filled_frame_receiver: Receiver<Box<[u8]>>,
    shutdown_requested: Arc<Mutex<bool>>,
    key_sender: Sender<u8>,

    screen_data: Vec<u8>,
    current_mode: ScreenMode,
    io_data_receiver: Receiver<ScreenSetting>,
    settings: [u32; 4],
}


impl Screen {
    pub fn new(
        filled_frame_receiver: Receiver<Box<[u8]>>,
        shutdown_requested: Arc<Mutex<bool>>,
        key_sender: Sender<u8>,
        io_data_receiver: Receiver<ScreenSetting>
    ) -> Self {
        Screen {
            window: None,
            pixels: None,
            filled_frame_receiver,
            shutdown_requested,
            key_sender,

            screen_data: vec![],
            current_mode: ScreenMode::Ascii8,
            io_data_receiver,
            settings: [0, 0xFFFFFFFF, 0xFF000000, 0]
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

        let mut pixels = Pixels::new(ASCII_SCREEN_WIDTH *8, ASCII_SCREEN_HEIGHT *8, surface_texture).unwrap();
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

                while let Ok(setting) = self.io_data_receiver.try_recv() {
                    if setting.option == 0 {
                        // Change mode
                        match setting.value {
                            0 => {
                                self.current_mode = ScreenMode::Ascii8;
                                let Some(pixels) = &mut self.pixels else { return; };
                                pixels.resize_buffer(ASCII_SCREEN_WIDTH *8, ASCII_SCREEN_HEIGHT *8)
                                    .expect("Could not resize pixels buffer");
                            },
                            1 => {
                                self.current_mode = ScreenMode::Ascii32;
                                let Some(pixels) = &mut self.pixels else { return; };
                                pixels.resize_buffer(ASCII_SCREEN_WIDTH *8, ASCII_SCREEN_HEIGHT *8)
                                    .expect("Could not resize pixels buffer");
                            }
                            3 => {
                                self.current_mode = ScreenMode::Pixel32;
                                let Some(pixels) = &mut self.pixels else { return; };
                                pixels.resize_buffer(SCREEN_WIDTH, SCREEN_HEIGHT)
                                    .expect("Could not resize pixels buffer");
                            },
                            _ => panic!("Invalid screen mode received.")
                        }
                    } else {
                        self.settings[setting.option as usize - 1] = setting.value;
                    }
                }


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
                        let frame_len = pixels.frame().len();
                        if self.screen_data.len() >= frame_len {
                            let start = self.settings[0] as usize;
                            pixels.frame_mut().copy_from_slice(&self.screen_data.as_slice()[start..start+frame_len]);
                        }
                    }
                    ScreenMode::Ascii8 => {
                        let frame = pixels.frame_mut();
                        render_ascii8(frame, self.settings[1], self.settings[2], self.screen_data.as_slice());
                    }
                    ScreenMode::Ascii32 => {
                        let frame = pixels.frame_mut();
                        render_ascii_32(frame, self.screen_data.as_slice());
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
                } else {
                    if let Some(key) = match &event.logical_key {
                        Key::Named(NamedKey::ArrowLeft) => Some(37),
                        Key::Named(NamedKey::ArrowUp) => Some(38),
                        Key::Named(NamedKey::ArrowRight) => Some(39),
                        Key::Named(NamedKey::ArrowDown) => Some(40),
                        Key::Named(NamedKey::Backspace) => Some(8),
                        _ => None,
                    } {
                        self.key_sender.send(key).expect("Couldnt send character");
                    };
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
    const CHAR_WIDTH: usize = 8;
    const CHAR_HEIGHT: usize = 8;

    let mut curr_x = 0;
    let mut curr_y = 0;

    for d in data {
        let c = *d as char;
        let bitmap = get_char_bitmap(c);

        for (y, row) in bitmap.iter().enumerate() {
            for x in 0..CHAR_WIDTH {
                let active = row >> x & 1 == 1;

                let color = if active { foreground_color } else { background_color };

                let index =
                    (
                        (curr_x + x) +
                        (curr_y + y) * ASCII_SCREEN_WIDTH as usize * 8
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

        curr_x += CHAR_WIDTH;
        if curr_x + CHAR_WIDTH > ASCII_SCREEN_WIDTH as usize * CHAR_WIDTH {
            curr_x = 0;
            curr_y += CHAR_HEIGHT;
        }
    }
}

fn render_ascii_32(frame: &mut [u8], data: &[u8]) {
    const CHAR_WIDTH: usize = 8;
    const CHAR_HEIGHT: usize = 8;
    const BYTES_PER_CHAR: usize = 4;

    let mut curr_x = 0;
    let mut curr_y = 0;

    for i in 0..data.len()/BYTES_PER_CHAR {
        let idx = i*BYTES_PER_CHAR;
        if data.len() < idx { return; }

        let foreground_rg = data[idx+2];
        let split_br = data[idx+1];
        let background_gb = data[idx];

        // Now foreground rg has 0xRG
        // Split br has 0xBR
        // background gb has 0xGB

        let small_foreground_color: u32 =
                (((foreground_rg as u32) & 0xF) << 4) |
                ((foreground_rg as u32) >> 4) |
                (((split_br as u32) & 0xF0) << 4);

        let small_background_color: u32 =
            (((background_gb as u32) & 0xF) << 8) |
            ((background_gb as u32) & 0xF0) |
            ((split_br as u32) & 0xF);

        // Now foreground color has 0xBGR and needs 0xAABBGGRR, alpha as 0xFF, same for background
        let mut foreground_color: u32 = 0;
        let mut background_color: u32 = 0;

        for n in 0..12 {
            let foreground_bit = ((small_foreground_color >> n) & 0b1) * 0b11;
            foreground_color |= foreground_bit << 2*n;

            let background_bit = ((small_background_color >> n) & 0b1) * 0b11;
            background_color |= background_bit << 2*n;
        }

        foreground_color = foreground_color | 0xFF << 24;
        background_color = background_color | 0xFF << 24;


        let c = data[idx+3] as char;
        let bitmap = get_char_bitmap(c);

        for (y, row) in bitmap.iter().enumerate() {
            for x in 0..CHAR_WIDTH {
                let active = row >> x & 1 == 1;

                let color = if active { foreground_color } else { background_color };

                let index = ((curr_x + x) +
                            (curr_y + y) * ASCII_SCREEN_WIDTH as usize * 8
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

        curr_x += CHAR_WIDTH;
        if curr_x + CHAR_WIDTH > ASCII_SCREEN_WIDTH as usize * CHAR_WIDTH {
            curr_x = 0;
            curr_y += CHAR_HEIGHT;
        }
    }
}


enum ScreenMode {
    Ascii8,
    Ascii32,
    Pixel32,
}


pub struct ScreenSetting {
    pub option: u8,
    pub value: u32,
}

fn get_char_bitmap(c: char) -> [u8; 8] {
    let val = c as u8;
    match val {
        0 => [0; 8],
        1..32 => font8x8::legacy::BLOCK_LEGACY[val as usize - 1],
        32..127 => font8x8::legacy::BASIC_LEGACY[val as usize],
        127 => font8x8::legacy::BLOCK_LEGACY[31],
        128..=u8::MAX => font8x8::legacy::BOX_LEGACY[val as usize - 128],
    }
}