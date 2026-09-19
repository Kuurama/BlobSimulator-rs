use std::f32::consts::PI;

pub struct Sensor {
    size: u8,
    angle_spacing: f32,
    offset_distance: u8,
}

impl Default for Sensor {
    fn default() -> Self {
        Self {
            size: 2,
            angle_spacing: PI / 6f32,
            offset_distance: 20,
        }
    }
}

impl Sensor {
    pub const fn angle_spacing(&self) -> f32 {
        self.angle_spacing
    }

    pub const fn size(&self) -> u8 {
        self.size
    }

    pub const fn offset_distance(&self) -> u8 {
        self.offset_distance
    }
}
