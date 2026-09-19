use crate::blob::Blob;
use crate::blob::group::BlobGroupError::{CountTooBig, RadiusTooLarge};
use crate::blob::position::Position;
use crate::{SIM_HEIGHT, SIM_WIDTH};
use num_traits::ToPrimitive;
use std::f32::consts::{GOLDEN_RATIO, TAU};
use thiserror::Error;

#[derive(Default)]
pub struct BlobGroup {
    pub blobs: Vec<Blob>,
}

#[derive(Error, Debug)]
pub enum BlobGroupError {
    #[error("Radius {radius} exceeds the maximum allowed radius {maximum}")]
    RadiusTooLarge { radius: u16, maximum: u16 },
    #[error("The count exceed f32")]
    CountTooBig,
}

impl BlobGroup {
    const GOLDEN_ANGLE: f32 = TAU / (GOLDEN_RATIO * GOLDEN_RATIO);

    /// # Errors
    /// Returns a [`RadiusTooLarge`] if the circle can't fit inside the simulation (from the center)
    pub fn create_in_circle(count: u32, radius: u16) -> Result<Self, BlobGroupError> {
        let max_radius = SIM_WIDTH.get().min(SIM_HEIGHT.get()) / 2;
        if radius > max_radius {
            return Err(RadiusTooLarge {
                radius,
                maximum: max_radius,
            });
        }

        let center = Position::default();
        let radius = f32::from(radius);
        let count_f32 = count.to_f32().ok_or(CountTooBig)?;

        Ok(Self {
            blobs: (0..count)
                .filter_map(|i| {
                    let i = i.to_f32()?;
                    let angle = i * Self::GOLDEN_ANGLE;
                    let distance = radius * ((i + 0.5) / count_f32).sqrt();

                    let (angle_sin, angle_cos) = angle.sin_cos();
                    Some(Blob {
                        position: Position {
                            x: center.x + distance * angle_cos,
                            y: center.y + distance * angle_sin,
                        },
                        ..Default::default()
                    })
                })
                .collect(),
        })
    }
}
