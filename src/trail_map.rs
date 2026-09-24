use crate::blob::{Blob, Position};
use crate::config::{SIM_HEIGHT, SIM_SIZE, SIM_WIDTH};
use num_traits::{MulAdd, ToPrimitive};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use thiserror::Error;

pub struct TrailMap {
    buffer: Box<[u32]>,
    evaporate_next: bool,
}

/// Sample count and channel sums for a pixel's blur neighborhood.
#[derive(Default)]
struct NeighborhoodTotals {
    sample_count: u16,
    red: u16,
    green: u16,
    blue: u16,
}

impl Default for TrailMap {
    fn default() -> Self {
        Self {
            buffer: vec![0; SIM_SIZE].into_boxed_slice(),
            evaporate_next: true,
        }
    }
}

#[derive(Error, Debug)]
#[error("Blob is outside the simulation bounds")]
pub struct BlobOutsideSimulationError;

impl TrailMap {
    pub fn pixels(&self) -> &[u32] {
        self.buffer.as_ref()
    }

    /// Writes each blob's displayed color at its position.
    ///
    /// # Errors
    /// Returns an error at the first non-finite or out-of-bounds position.
    /// Colors written for preceding blobs remain in the trail map.
    pub fn deposit(&mut self, blobs: &[Blob]) -> Result<(), BlobOutsideSimulationError> {
        for blob in blobs {
            let Position { x, y } = blob.position();
            if !(0.0..f32::from(SIM_WIDTH.get())).contains(x)
                || !(0.0..f32::from(SIM_HEIGHT.get())).contains(y)
            {
                return Err(BlobOutsideSimulationError);
            }
            let x = x.to_usize().ok_or(BlobOutsideSimulationError)?;
            let y = y.to_usize().ok_or(BlobOutsideSimulationError)?;

            if let Some(pixel) = self
                .buffer
                .get_mut(y.mul_add(usize::from(SIM_WIDTH.get()), x))
            {
                *pixel = blob.displayed_color();
            } else {
                return Err(BlobOutsideSimulationError);
            }
        }

        Ok(())
    }

    /// Blurs and evaporates the trail map into `destination`.
    ///
    /// Processes pixels in parallel using Rayon and retains the result.
    /// Evaporation removes one unit per channel every other call, starting
    /// with the first call.
    ///
    /// # Panics
    /// Panics if `destination` does not have the same length as the trail map.
    pub fn blur_and_evaporate_into(&mut self, destination: &mut [u32]) {
        assert_eq!(destination.len(), self.buffer.len());
        let evaporation = u16::from(self.evaporate_next);
        destination
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, pixel)| {
                *pixel = self.blurred_and_evaporated_pixel(i, evaporation);
            });

        self.buffer.copy_from_slice(destination);
        self.evaporate_next = !self.evaporate_next;
    }

    #[allow(
        clippy::arithmetic_side_effects,
        reason = "The simulation width is nonzero, so division and remainder are defined"
    )]
    fn blurred_and_evaporated_pixel(&self, i: usize, evaporation: u16) -> u32 {
        let width = usize::from(SIM_WIDTH.get());
        let height = usize::from(SIM_HEIGHT.get());
        let x = i % width;
        let y = i / width;
        // Clip the neighborhood in two dimensions to avoid crossing row boundaries.
        let columns = x.saturating_sub(1)..=x.saturating_add(1).min(width.saturating_sub(1));
        let rows = y.saturating_sub(1)..=y.saturating_add(1).min(height.saturating_sub(1));

        let totals = rows
            .flat_map(|row| {
                columns
                    .clone()
                    .filter_map(move |column| self.buffer.get(row.mul_add(width, column)))
            })
            .fold(NeighborhoodTotals::default(), |acc, pixel| {
                let [_, red, green, blue] = pixel.to_be_bytes();
                NeighborhoodTotals {
                    sample_count: acc.sample_count.saturating_add(1),
                    red: acc.red.saturating_add(u16::from(red)),
                    green: acc.green.saturating_add(u16::from(green)),
                    blue: acc.blue.saturating_add(u16::from(blue)),
                }
            });

        u32::from_be_bytes([
            0,
            averaged_channel(totals.red, totals.sample_count, evaporation),
            averaged_channel(totals.green, totals.sample_count, evaporation),
            averaged_channel(totals.blue, totals.sample_count, evaporation),
        ])
    }
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "The neighborhood includes the center pixel, so the count can't be zero"
)]
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "An average of u8 channels is at most 255, evaporation only decreases it"
)]
const fn averaged_channel(total: u16, count: u16, evaporation: u16) -> u8 {
    (total / count).saturating_sub(evaporation) as u8
}
