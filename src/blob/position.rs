use crate::{SIM_HEIGHT, SIM_WIDTH};

pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: f32::from(SIM_WIDTH.get()) / 2f32,
            y: f32::from(SIM_HEIGHT.get()) / 2f32,
        }
    }
}
