use crate::emulator::emulator_controller::SCREEN_RES;
use pixels::{Pixels, SurfaceTexture};
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
            current_mode: ScreenMode::Ascii8 { foreground_color: 0xFFFFFFFF, background_color: 0x00 },
        }
    }
}

impl ApplicationHandler for Screen {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }

        let window_attributes = WindowAttributes::default();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let physical_window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(
            physical_window_size.width,
            physical_window_size.height,
            window.clone(),
        );

        let pixels = Pixels::new(SCREEN_RES*4, SCREEN_RES*3, surface_texture).unwrap();
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
                let mut newest_frame = None;
                while let Ok(new_pixels) = self.filled_frame_receiver.try_recv() {
                    newest_frame = Some(new_pixels);
                }

                if newest_frame.is_none() { return; }

                if let Some(new_data) = newest_frame {
                    self.screen_data = new_data.to_vec();
                }

                let Some(pixels) = self.pixels.as_mut() else { return; };
                match self.current_mode {
                    ScreenMode::Pixel32 => {
                        pixels.frame_mut().copy_from_slice(self.screen_data.as_slice());
                        pixels.render().unwrap();
                    }
                    ScreenMode::Ascii8 { foreground_color, background_color } => {
                        let frame = pixels.frame_mut();

                        render_ascii8(frame, foreground_color, background_color, self.screen_data.as_slice());
                    }
                    ScreenMode::Ascii32 => {

                    }
                }


                if let Some(window) = &self.window {
                    std::thread::sleep(Duration::from_millis(33));
                    window.request_redraw();
                }


                if self.shutdown_requested.lock().unwrap().clone() {
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                if event.state == winit::event::ElementState::Released { return; }
                if let Some(text) = event.text {
                    for c in text.as_bytes() {
                        self.key_sender.send(*c).expect("Couldnt send character");
                    }
                }

            }
            _ => {}
        }
    }
}


fn render_ascii8(_frame: &mut [u8], _foreground_color: u32, _background_color: u32, _data: &[u8]) {

}


enum ScreenMode {
    Ascii8 {
        foreground_color: u32,
        background_color: u32,
    },
    Ascii32,
    Pixel32,
}