use pixels::{Pixels, SurfaceTexture};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Screen {
    pub window: Option<Arc<Window>>,
    pub pixels: Option<Pixels<'static>>,
    pub filled_frame_receiver: Receiver<Box<[u8]>>,
    pub shutdown_requested: Arc<Mutex<bool>>,
    pub key_sender: Sender<u8>,
}


impl Screen {
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

        let res = 32;
        let pixels = Pixels::new(res*4, res*3, surface_texture).unwrap();
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
                if let (Some(window), Some(pixels)) = (&self.window, &mut self.pixels) {
                    let mut newest_frame = None;
                    while let Ok(new_pixels) = self.filled_frame_receiver.try_recv() {
                        newest_frame = Some(new_pixels);
                    }
                    if let Some(new_pixels) = newest_frame {
                        pixels.frame_mut().copy_from_slice(&new_pixels);
                    }

                    pixels.render().unwrap();

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