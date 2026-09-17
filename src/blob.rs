use self::position::Position;
use self::settings::BlobSettings;
use crate::{SIM_HEIGHT, SIM_SIZE, SIM_WIDTH};
use num_traits::ToPrimitive;
use rand::Rng;
use std::f32::consts::PI;

mod position;
mod sensor;
mod settings;

#[derive(Default)]
pub struct Blob {
    position: Position,
    angle: f32,
    settings: BlobSettings,
}

impl Blob {
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

        if pos_x < 0f32
            || pos_x >= f32::from(SIM_WIDTH)
            || pos_y < 0f32
            || pos_y >= f32::from(SIM_HEIGHT)
        {
            pos_x = (f32::from(SIM_WIDTH) - 0.01f32).min(0f32.max(pos_x));
            pos_y = (f32::from(SIM_HEIGHT) - 0.01f32).min(0f32.max(pos_y));

            self.angle = f32::from_bits(rng.next_u32()) * 2f32 * PI;
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

        let random_steer_strength = f32::from_bits(rng.next_u32());

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

                if x >= usize::from(SIM_WIDTH) || y >= usize::from(SIM_HEIGHT) {
                    return None;
                }

                let index = y.checked_mul(usize::from(SIM_WIDTH))?.checked_add(x)?;
                trail_map.get(index)?.to_f32()
            })
            .reduce(|a, b| a + b)
    }
}
