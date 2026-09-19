use super::sensor::Sensor;

pub struct BlobSettings {
    color: [u8; 3],
    speed: f32,
    turn_speed: f32,
    sensor: Sensor,
}

impl Default for BlobSettings {
    fn default() -> Self {
        Self {
            color: [0, 191, 255],
            speed: 3.0,
            turn_speed: 1.5,
            sensor: Sensor::default(),
        }
    }
}

impl BlobSettings {
    pub const fn color(&self) -> [u8; 3] {
        self.color
    }

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
