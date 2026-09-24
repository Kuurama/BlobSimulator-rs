//! Compile-time simulation dimensions and initial population.

use std::num::NonZero;

#[allow(clippy::unwrap_used, reason = "checked at compile time")]
const WINDOW_WIDTH: NonZero<u16> = NonZero::new(256).unwrap();

#[allow(clippy::unwrap_used, reason = "checked at compile time")]
const WINDOW_HEIGHT: NonZero<u16> = NonZero::new(144).unwrap();

#[allow(clippy::unwrap_used, reason = "checked at compile time")]
const SIM_SCALE_MULTIPLIER: NonZero<u16> = NonZero::new(10).unwrap();
pub const SIM_WIDTH: NonZero<u16> = WINDOW_WIDTH.saturating_mul(SIM_SCALE_MULTIPLIER);
pub const SIM_HEIGHT: NonZero<u16> = WINDOW_HEIGHT.saturating_mul(SIM_SCALE_MULTIPLIER);

#[allow(clippy::as_conversions, reason = "u16 always fits in usize")]
pub const SIM_SIZE: usize = (SIM_WIDTH.get() as usize) * (SIM_HEIGHT.get() as usize);

pub const BLOB_COUNT: u32 = 50_000;
pub const BLOB_SPAWN_RADIUS: u16 = 250;
