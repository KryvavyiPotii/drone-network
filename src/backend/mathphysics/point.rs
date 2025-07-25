use serde::{Deserialize, Serialize};

use derive_more::{Add, Div, DivAssign, Mul, MulAssign, Sub};

use super::Position;


#[derive(
    Copy, Clone, PartialEq, Add, Sub, Mul, MulAssign, Div, DivAssign, Debug, 
    Default, Serialize, Deserialize,
)]
pub struct Point3D { 
    pub x: f32, 
    pub y: f32, 
    pub z: f32, 
}

impl Point3D {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<(f32, f32, f32)> for Point3D {
    fn from(value: (f32, f32, f32)) -> Self {
        Self { 
            x: value.0, 
            y: value.1, 
            z: value.2 
        } 
    }
}

impl From<Point3D> for (f64, f64, f64) {
    fn from(point: Point3D) -> Self {
        (
            f64::from(point.x), 
            f64::from(point.y), 
            f64::from(point.z), 
        )
    }
}

impl Position for Point3D {
    fn position(&self) -> &Self {
        self
    }
}
