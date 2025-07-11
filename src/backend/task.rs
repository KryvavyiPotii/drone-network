use serde::Serialize;

use super::mathphysics::Point3D;

pub use scenario::Scenario;


pub mod scenario;


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub enum TaskType {
    Attack,    
    Reconnect,
    Reposition,
    #[default]
    Undefined,
}


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Task {
    task_type: TaskType,
    destination: Option<Point3D>,
}

impl Task {
    #[must_use]
    pub fn new(
        task_type: TaskType,
        destination: Option<Point3D>,
    ) -> Self {
        Self { task_type, destination }
    }

    #[must_use]
    pub fn task_type(&self) -> TaskType {
        self.task_type
    }

    #[must_use]
    pub fn destination(&self) -> Option<&Point3D> {
        self.destination.as_ref()
    }
}
