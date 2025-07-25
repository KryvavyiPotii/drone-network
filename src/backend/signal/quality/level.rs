use derive_more::Display;
use serde::{Deserialize, Serialize};


#[derive(
    Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Debug, Display, 
    Serialize, Deserialize
)]
pub enum SignalLevel {
    #[default]
    #[display("Black")]
    Black,  // (almost) no signal
    #[display("Red")]
    Red,    // signal level is critically low
    #[display("Yellow")]
    Yellow, // signal level is decent
    #[display("Green")]
    Green   // signal level is good
}
