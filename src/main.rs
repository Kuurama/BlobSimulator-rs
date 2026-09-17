mod blob;

use crate::blob::Blob;
use softbuffer::Surface;
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::{ActiveEventLoop, ControlFlow, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};

const WINDOW_WIDTH: u16 = 256;
const WINDOW_HEIGHT: u16 = 144;

#[allow(
    clippy::as_conversions,
    reason = "u16 values are exactly representable as f32"
)]
const SCREEN_RATIO: f32 = (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32);

const SIM_SCALE_MULTIPLIER: u16 = 6;
const SIM_WIDTH: u16 = WINDOW_WIDTH * SIM_SCALE_MULTIPLIER;
const SIM_HEIGHT: u16 = WINDOW_HEIGHT * SIM_SCALE_MULTIPLIER;

#[allow(clippy::as_conversions, reason = "u16 always fits in usize")]
const SIM_SIZE: usize = (SIM_WIDTH as usize) * (SIM_HEIGHT as usize);

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(App::default())?;

    Ok(())
}

#[derive(Default)]
struct App {
    surface: Option<Surface<OwnedDisplayHandle, Box<dyn Window>>>,
    blobs: [Blob; 32]
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &(dyn ActiveEventLoop + 'static)) {
        if self.surface.is_some() {
            return;
        }

        let (Ok(window), Ok(context)) = (
            event_loop.create_window(WindowAttributes::default().with_title("BlobSimulator-rs")),
            softbuffer::Context::new(event_loop.owned_display_handle()),
        ) else {
            eprintln!("Failed to create window and/or context");
            event_loop.exit();
            return;
        };

        self.surface = match Surface::new(&context, window) {
            Ok(surface) => Some(surface),
            Err(error) => {
                eprintln!("Failed to create Surface {error}");
                event_loop.exit();
                return;
            }
        };
    }

    fn window_event(
        &mut self,
        event_loop: &(dyn ActiveEventLoop + 'static),
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(size) => {
                let Some(surface) = self.surface.as_mut() else {
                    return;
                };

                if let (Some(width), Some(height)) = (
                    std::num::NonZeroU32::new(size.width),
                    std::num::NonZeroU32::new(size.height),
                ) {
                    if let Err(error) = surface.resize(width, height) {
                        eprintln!("Failed to resize surface {error}");
                        event_loop.exit();
                        return;
                    }

                    surface.window().request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(surface) = self.surface.as_mut() else {
                    return;
                };

                surface.window().pre_present_notify();

                let mut buffer = match surface.buffer_mut() {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        eprintln!("Failed to get buffer {error}");
                        event_loop.exit();
                        return;
                    }
                };

                buffer.fill(0x0020_3040);

                if let Err(error) = buffer.present() {
                    eprintln!("Failed to present {error}");
                    event_loop.exit();
                    return;
                }

                // For now, the simulation loop is gonna be tied to redraw requests.
                surface.window().request_redraw();
            }
            _ => {}
        }
    }
}
