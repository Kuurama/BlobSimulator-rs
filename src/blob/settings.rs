use super::sensor::Sensor;

pub struct BlobSettings {
    speed: f32,
    turn_speed: f32,
    sensor: Sensor,
}

impl Default for BlobSettings {
    fn default() -> Self {
        Self {
            speed: 1.0,
            turn_speed: 0.4,
            sensor: Sensor::default(),
        }
    }
}

impl BlobSettings {
    pub const fn speed(&self) -> f32 {
        self.speed
    }

    pub const fn turn_speed(&self) -> f32 {
        self.turn_speed
    }

    pub const fn sensor(&self) -> &Sensor {
        &self.sensor
    }
}
