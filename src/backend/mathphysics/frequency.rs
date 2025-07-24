use serde::{Deserialize, Serialize};


// The representation type needs to be updated if the `Megahertz` type is 
// changed.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Frequency {
    Control = 2_400,
    GPS     = 1_575,
}
