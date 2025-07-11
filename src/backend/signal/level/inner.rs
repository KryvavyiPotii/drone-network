use std::fmt;

use serde::Serialize;

use crate::backend::signal::strength::{
    MAX_BLACK_SIGNAL_STRENGTH, MAX_RED_SIGNAL_STRENGTH, 
    MAX_YELLOW_SIGNAL_STRENGTH, SignalStrength, 
};


const DISPLAY_GREEN_LEVEL: &str  = "Green";
const DISPLAY_YELLOW_LEVEL: &str = "Yellow";
const DISPLAY_RED_LEVEL: &str    = "Red";
const DISPLAY_BLACK_LEVEL: &str  = "Black";


#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize)]
pub enum SignalLevelInner {
    Black(SignalStrength),  // (almost) no signal
    Red(SignalStrength),    // signal level is critically low
    Yellow(SignalStrength), // signal level is decent
    Green(SignalStrength)   // signal level is good
}

impl Default for SignalLevelInner {
    fn default() -> Self {
        Self::Black(MAX_BLACK_SIGNAL_STRENGTH)
    }
}

impl From<f32> for SignalLevelInner {
    fn from(value: f32) -> Self {
        Self::from(SignalStrength::new(value))
    }
}

impl From<SignalStrength> for SignalLevelInner {
    fn from(signal_strength: SignalStrength) -> Self {
        if signal_strength > MAX_YELLOW_SIGNAL_STRENGTH {
            Self::Green(signal_strength)
        } else if signal_strength > MAX_RED_SIGNAL_STRENGTH {
            Self::Yellow(signal_strength)
        } else if signal_strength > MAX_BLACK_SIGNAL_STRENGTH {
            Self::Red(signal_strength)
        } else {
            Self::Black(signal_strength)
        }
    }
}

impl fmt::Display for SignalLevelInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (color_str, strength_value) = match self {
            Self::Green(strength)  => (DISPLAY_GREEN_LEVEL, strength.value()),
            Self::Yellow(strength) => (DISPLAY_YELLOW_LEVEL, strength.value()),
            Self::Red(strength)    => (DISPLAY_RED_LEVEL, strength.value()),
            Self::Black(strength)  => (DISPLAY_BLACK_LEVEL, strength.value()),
        };

        write!(f, "{}({})", color_str, strength_value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn signal_level_color_from_strength() {
        assert_eq!(
            SignalLevelInner::from(-6.0),
            SignalLevelInner::Black(SignalStrength::new(-6.0))
        );
        assert_eq!(
            SignalLevelInner::from(0.1),
            SignalLevelInner::Black(SignalStrength::new(0.1))
        );
        assert_eq!(
            SignalLevelInner::from(MAX_YELLOW_SIGNAL_STRENGTH),
            SignalLevelInner::Yellow(MAX_YELLOW_SIGNAL_STRENGTH)
        );
    }
}
