pub mod blob;

use crate::blob::{BlobGroup, Position};
use num_traits::{MulAdd, ToPrimitive};
use softbuffer::Surface;
use std::error::Error;
use std::num::NonZero;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::{ActiveEventLoop, ControlFlow, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};

#[allow(clippy::unwrap_used, reason = "checked at compile time")]
const WINDOW_WIDTH: NonZero<u16> = NonZero::new(256).unwrap();

#[allow(clippy::unwrap_used, reason = "checked at compile time")]
const WINDOW_HEIGHT: NonZero<u16> = NonZero::new(144).unwrap();

/*#[allow(
    clippy::as_conversions,
    reason = "u16 values are exactly representable as f32"
)]
const SCREEN_RATIO: f32 = (WINDOW_WIDTH.get() as f32) / (WINDOW_HEIGHT.get() as f32);*/

#[allow(clippy::unwrap_used, reason = "checked at compile time")]
const SIM_SCALE_MULTIPLIER: NonZero<u16> = NonZero::new(6).unwrap();
const SIM_WIDTH: NonZero<u16> = WINDOW_WIDTH.saturating_mul(SIM_SCALE_MULTIPLIER);
const SIM_HEIGHT: NonZero<u16> = WINDOW_HEIGHT.saturating_mul(SIM_SCALE_MULTIPLIER);

#[allow(clippy::as_conversions, reason = "u16 always fits in usize")]
const SIM_SIZE: usize = (SIM_WIDTH.get() as usize) * (SIM_HEIGHT.get() as usize);

const BLOB_COUNT: u16 = 10_000;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(App::default())?;

    Ok(())
}

#[derive(Default)]
struct App {
    surface: Option<Surface<OwnedDisplayHandle, Box<dyn Window>>>,
    blob_group: BlobGroup,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &(dyn ActiveEventLoop + 'static)) {
        if self.surface.is_some() {
            return;
        }

        let (Ok(window), Ok(context)) = (
            event_loop.create_window(
                WindowAttributes::default()
                    .with_title("BlobSimulator-rs")
                    .with_surface_size(PhysicalSize::new(SIM_WIDTH.get(), SIM_HEIGHT.get()))
                    .with_resizable(false),
            ),
            softbuffer::Context::new(event_loop.owned_display_handle()),
        ) else {
            eprintln!("Failed to create window and/or context");
            event_loop.exit();
            return;
        };

        self.surface = match Surface::new(&context, window) {
            Ok(mut surface) => {
                if let Err(error) = surface.resize(SIM_WIDTH.into(), SIM_HEIGHT.into()) {
                    eprintln!("Couldn't resize Surface: {error}");
                    event_loop.exit();
                    return;
                }

                Some(surface)
            }
            Err(error) => {
                eprintln!("Failed to create Surface: {error}");
                event_loop.exit();
                return;
            }
        };

        self.blob_group = match BlobGroup::create_in_circle(BLOB_COUNT, 200) {
            Ok(group) => group,
            Err(error) => {
                eprintln!("Failed to create the blobs: {error}");
                event_loop.exit();
                return;
            }
        }
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
            WindowEvent::RedrawRequested => {
                let Some(surface) = self.surface.as_mut() else {
                    return;
                };

                surface.window().pre_present_notify();

                let mut buffer = match surface.buffer_mut() {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        eprintln!("Failed to get buffer: {error}");
                        event_loop.exit();
                        return;
                    }
                };

                for blob in &self.blob_group.blobs {
                    let Position { x, y } = blob.position();
                    let (Some(x), Some(y)) = (x.to_usize(), y.to_usize()) else {
                        continue;
                    };

                    if let Some(pixel) = buffer.get_mut(y.mul_add(usize::from(SIM_WIDTH.get()), x))
                    {
                        *pixel = 0x0020_3040;
                    } else {
                        eprintln!("Blob shouldn't go outside the Simulation bounds");
                        event_loop.exit();
                        return;
                    }
                }

                if let Err(error) = buffer.present() {
                    eprintln!("Failed to present: {error}");
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
