use mathphysics::{Meter, Millisecond};


pub mod connections;
pub mod device;
pub mod malware;
pub mod mathphysics;
pub mod networkmodel;
pub mod signal;
pub mod task;


pub const DESTINATION_RADIUS: Meter   = 5.0;
pub const ITERATION_TIME: Millisecond = 50;
