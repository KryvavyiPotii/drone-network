use serde::Serialize;

use super::mathphysics::Point3D;

pub use scenario::Scenario;


pub mod scenario;


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub enum Task {
    Attack(Point3D),
    Reconnect(Point3D),
    Reposition(Point3D),
    #[default]
    Undefined,
}
