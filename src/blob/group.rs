use crate::blob::Blob;
use crate::blob::group::BlobGroupError::RadiusTooLarge;
use crate::blob::position::Position;
use crate::config::{SIM_HEIGHT, SIM_WIDTH};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::f32::consts::{GOLDEN_RATIO, PI, TAU};
use thiserror::Error;

#[derive(Default)]
pub struct BlobGroup {
    blobs: Vec<Blob>,
}

#[derive(Error, Debug)]
pub enum BlobGroupError {
    #[error("Radius {radius} exceeds the maximum allowed radius {maximum}")]
    RadiusTooLarge { radius: u16, maximum: u16 },
}

impl BlobGroup {
    const GOLDEN_ANGLE: f32 = TAU / (GOLDEN_RATIO * GOLDEN_RATIO);

    #[must_use]
    pub fn blobs(&self) -> &[Blob] {
        self.blobs.as_ref()
    }

    /// Advances all blobs in parallel using Rayon.
    pub fn step(&mut self, trail_pixels: &[u32]) {
        self.blobs
            .par_iter_mut()
            .for_each_init(rand::rng, |rng, blob| blob.next_step(trail_pixels, rng));
    }

    /// # Errors
    /// Returns a [`RadiusTooLarge`] if the circle can't fit inside the simulation (from the center)
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "u32 fit in f32's range, rounding is acceptable"
    )]
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
        let count_f32 = count as f32;

        Ok(Self {
            blobs: (0..count)
                .map(|i| {
                    let i = i as f32;
                    let angle = i * Self::GOLDEN_ANGLE;
                    let distance = radius * ((i + 0.5) / count_f32).sqrt();

                    let (angle_sin, angle_cos) = angle.sin_cos();
                    Blob {
                        position: Position {
                            x: center.x + distance * angle_cos,
                            y: center.y + distance * angle_sin,
                        },
                        angle: (angle + PI).rem_euclid(TAU),
                        ..Default::default()
                    }
                })
                .collect(),
        })
    }
}
