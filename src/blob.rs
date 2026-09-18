use self::settings::BlobSettings;
use crate::{SIM_HEIGHT, SIM_SIZE, SIM_WIDTH};
use num_traits::ToPrimitive;
use rand::{Rng, RngExt};
use std::f32::consts::{PI, TAU};

mod group;
mod position;
mod sensor;
mod settings;

pub use group::BlobGroup;
pub use position::Position;

#[derive(Default)]
pub struct Blob {
    position: Position,
    angle: f32,
    settings: BlobSettings,
}

impl Blob {
    #[must_use]
    pub const fn position(&self) -> &Position {
        &self.position
    }

    pub fn next_step(&mut self, trail_map: &[u32; SIM_SIZE], rng: &mut impl Rng) {
        self.recalculate_angle(trail_map, rng);
        self.r#move(rng);
    }

    fn r#move(&mut self, rng: &mut impl Rng) {
        let (direction_y, direction_x) = self.angle.sin_cos();
        let (mut pos_x, mut pos_y) = (
            direction_x.mul_add(self.settings.speed(), self.position.x),
            direction_y.mul_add(self.settings.speed(), self.position.y),
        );

        let width = f32::from(SIM_WIDTH.get());
        let height = f32::from(SIM_HEIGHT.get());
        if pos_x < 0f32 || pos_x >= width || pos_y < 0f32 || pos_y >= height {
            pos_x = (width - 0.01f32).min(0f32.max(pos_x));
            pos_y = (height - 0.01f32).min(0f32.max(pos_y));

            self.angle = rng.random_range(0f32..TAU);
        }

        self.position.x = pos_x;
        self.position.y = pos_y;
    }

    fn recalculate_angle(&mut self, trail_map: &[u32; SIM_SIZE], rng: &mut impl Rng) {
        let (forward_weight, left_weight, right_weight) = (
            self.sense(trail_map, 0f32),
            self.sense(trail_map, self.settings.sensor().angle_spacing()),
            self.sense(trail_map, -self.settings.sensor().angle_spacing()),
        );

        let random_steer_strength = rng.random::<f32>();

        self.angle += match (forward_weight, left_weight, right_weight) {
            (None, None, None) => PI,
            (f, l, r) if f > l && f > r => 0f32,
            (f, l, r) if f < l && f < r => {
                (random_steer_strength - 0.5f32) * 2f32 * self.settings.turn_speed()
            }
            (_, l, r) if r > l => -random_steer_strength * self.settings.turn_speed(),
            (_, l, r) if l > r => random_steer_strength * self.settings.turn_speed(),
            _ => 0f32,
        };
    }

    fn sense(&self, trail_map: &[u32; SIM_SIZE], angle_offset: f32) -> Option<f32> {
        let sensor_angle = self.angle + angle_offset;
        let (direction_y, direction_x) = sensor_angle.sin_cos();

        (0..self.settings.sensor().size())
            .filter_map(|i| {
                let distance = f32::from(self.settings.sensor().offset_distance()) + f32::from(i);
                let (x, y) = (
                    direction_x
                        .mul_add(distance, self.position.x)
                        .floor()
                        .to_usize()?,
                    direction_y
                        .mul_add(distance, self.position.y)
                        .floor()
                        .to_usize()?,
                );

                if x >= usize::from(SIM_WIDTH.get()) || y >= usize::from(SIM_HEIGHT.get()) {
                    return None;
                }

                let index = y
                    .checked_mul(usize::from(SIM_WIDTH.get()))?
                    .checked_add(x)?;
                trail_map.get(index)?.to_f32()
            })
            .reduce(|a, b| a + b)
    }
}
