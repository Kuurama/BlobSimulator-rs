use crate::blob::Blob;
use crate::blob::position::Position;
use crate::{SIM_HEIGHT, SIM_WIDTH};
use std::f32::consts::{GOLDEN_RATIO, TAU};
use thiserror::Error;

#[derive(Default)]
pub struct BlobGroup {
    pub blobs: Vec<Blob>,
}

#[derive(Debug, Error)]
#[error("radius {radius} exceeds the maximum allowed radius {maximum}")]
pub struct RadiusTooLarge {
    pub radius: u16,
    pub maximum: u16,
}

impl BlobGroup {
    const GOLDEN_ANGLE: f32 = TAU / (GOLDEN_RATIO * GOLDEN_RATIO);

    /// # Errors
    /// Returns a [`RadiusTooLarge`] if the circle can't fit inside the simulation (from the center)
    pub fn create_in_circle(count: u16, radius: u16) -> Result<Self, RadiusTooLarge> {
        let max_radius = SIM_WIDTH.get().min(SIM_HEIGHT.get()) / 2;
        if radius > max_radius {
            return Err(RadiusTooLarge {
                radius,
                maximum: max_radius,
            });
        }

        let center = Position::default();
        let radius = f32::from(radius);

        Ok(Self {
            blobs: (0..count)
                .map(|i| {
                    let i = f32::from(i);
                    let angle = i * Self::GOLDEN_ANGLE;
                    let distance = radius * ((i + 0.5) / f32::from(count)).sqrt();

                    let (angle_sin, angle_cos) = angle.sin_cos();
                    Blob {
                        position: Position {
                            x: center.x + distance * angle_cos,
                            y: center.y + distance * angle_sin,
                        },
                        ..Default::default()
                    }
                })
                .collect(),
        })
    }
}
