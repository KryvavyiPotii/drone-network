use serde::{Deserialize, Serialize};

use super::mathphysics::Point3D;

pub use scenario::Scenario;


pub mod scenario;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Task {
    Attack(Point3D),    
    Reconnect(Point3D),  // Moving to a point to receive a control signal
    Reposition(Point3D),
    Undefined,
}
