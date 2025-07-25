use super::ITERATION_TIME;


pub use frequency::Frequency;
pub use point::Point3D;
pub use unit::*;
pub use vector::Vector3D;


pub mod frequency;
pub mod point;
pub mod unit;
pub mod vector;


#[must_use]
pub fn delay_to(distance: Meter, multiplier: f32) -> Millisecond {    
    if multiplier == 0.0 {
        return 0;
    }

    let delay = time_in_millis_from_distance_and_speed(
        distance * multiplier as Meter,
        kmps_to_mpms(SPEED_OF_LIGHT) 
    );
    let reminder = delay % ITERATION_TIME;
    
    delay - reminder
}

#[must_use]
pub fn equation_of_motion_1d(
    start_position: Meter,
    velocity: MeterPerSecond,
    time: Second
) -> Meter {
    velocity.mul_add(time, start_position)
}

#[must_use]
pub fn equation_of_motion_3d(
    start_position: &Point3D,
    velocity: &Point3D,
    time: Second
) -> Point3D {
    Point3D::new(
        equation_of_motion_1d(start_position.x, velocity.x, time),
        equation_of_motion_1d(start_position.y, velocity.y, time),
        equation_of_motion_1d(start_position.z, velocity.z, time),
    )
}


pub trait Position {
    fn position(&self) -> &Point3D;

    fn distance_to<P: Position>(&self, other: &P) -> f32 {
        let vector = Vector3D::new(*self.position(), *other.position());
        
        vector.size()
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn distance_to_another_point() {
        let origin = Point3D::default();
        let some_point = Point3D::new(5.0, 0.0, 0.0);

        assert_eq!(0.0, origin.distance_to(&origin));
        assert_eq!(5.0, origin.distance_to(&some_point));
    }
}
