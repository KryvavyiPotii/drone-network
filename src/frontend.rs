use super::backend::mathphysics::Millisecond;


pub mod cli;
pub mod examples;
pub mod player;
pub mod renderer;


pub const MALWARE_INFECTION_DELAY: Millisecond      = 500;
pub const MALWARE_SPREAD_DELAY: Option<Millisecond> = Some(200);
