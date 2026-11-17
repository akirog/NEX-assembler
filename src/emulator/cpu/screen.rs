use crate::emulator::emulator_controller::SCREEN_AREA;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Screen {
    pub window: Option<Arc<Window>>,
    pub pixels: Option<Pixels<'static>>,
    pub filled_frame_receiver: Receiver<Box<[u8]>>,
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
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(pixels)) = (&self.window, &mut self.pixels) {
                    if let Ok(new_pixels) = self.filled_frame_receiver.try_recv() {
                        pixels.frame_mut().copy_from_slice(&new_pixels);
                    }

                    pixels.render().unwrap();

                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}