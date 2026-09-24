pub mod blob;
mod config;
mod trail_map;

use crate::blob::BlobGroup;
use crate::config::{BLOB_COUNT, BLOB_SPAWN_RADIUS, SIM_HEIGHT, SIM_WIDTH};
use crate::trail_map::TrailMap;
use softbuffer::Surface;
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::{ActiveEventLoop, ControlFlow, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};

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
    trail_map: TrailMap,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &(dyn ActiveEventLoop + 'static)) {
        if self.surface.is_some() {
            return;
        }

        let window = match event_loop.create_window(
            WindowAttributes::default()
                .with_title("BlobSimulator-rs")
                .with_surface_size(PhysicalSize::new(SIM_WIDTH.get(), SIM_HEIGHT.get()))
                .with_resizable(false),
        ) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("Failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        let context = match softbuffer::Context::new(event_loop.owned_display_handle()) {
            Ok(context) => context,
            Err(error) => {
                eprintln!("Failed to create rendering context: {error}");
                event_loop.exit();
                return;
            }
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

        self.blob_group = match BlobGroup::create_in_circle(BLOB_COUNT, BLOB_SPAWN_RADIUS) {
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

                self.blob_group.step(self.trail_map.pixels());
                if let Err(error) = self.trail_map.deposit(self.blob_group.blobs()) {
                    eprintln!("Failed to deposit blobs: {error}");
                    event_loop.exit();
                    return;
                }

                self.trail_map.blur_and_evaporate_into(&mut buffer);

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
