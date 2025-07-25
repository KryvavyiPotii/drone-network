use std::cmp::Ordering;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::backend::mathphysics::{Megahertz, Meter};


pub use level::*;
pub use strength::*;


pub mod level;
pub mod strength;


pub const BLACK_SIGNAL_QUALITY: SignalQuality  = SignalQuality {
    strength: MAX_BLACK_SIGNAL_STRENGTH,
    level: SignalLevel::Black,
};
pub const RED_SIGNAL_QUALITY: SignalQuality    = SignalQuality {
    strength: MAX_RED_SIGNAL_STRENGTH,
    level: SignalLevel::Red,
};
pub const YELLOW_SIGNAL_QUALITY: SignalQuality = SignalQuality {
    strength: MAX_YELLOW_SIGNAL_STRENGTH,
    level: SignalLevel::Yellow,
};
pub const GREEN_SIGNAL_QUALITY: SignalQuality  = SignalQuality {
    strength: GREEN_SIGNAL_STRENGTH,
    level: SignalLevel::Green,
};

const YELLOW_SIGNAL_ZONE_COEFFICIENT: f32 = 0.2;
const GREEN_SIGNAL_ZONE_COEFFICIENT: f32  = 0.1;


#[derive(
    Clone, Copy, Debug, Display, Default, PartialEq, Serialize, Deserialize
)]
#[display("{level}({strength})")]
pub struct SignalQuality {
    strength: SignalStrength,
    level: SignalLevel,
}

impl SignalQuality {
    #[must_use]
    pub fn from_area_radius(area_radius: Meter, frequency: Megahertz) -> Self {
        let tx_signal_strength = SignalStrength::from_area_radius(
            area_radius,
            frequency
        );

        Self::from(tx_signal_strength)
    }

    #[must_use]
    pub fn at_by_strength(
        &self, 
        frequency: Megahertz, 
        distance: Meter
    ) -> Self {
        Self::from(self.strength.at(frequency, distance))
    }
    
    #[must_use]
    pub fn at_by_level(&self, frequency: Megahertz, distance: Meter) -> Self {
        let radius = self.strength.area_radius_on(frequency); 

        if distance <= radius * GREEN_SIGNAL_ZONE_COEFFICIENT {
            *self
        } else if distance <= radius * YELLOW_SIGNAL_ZONE_COEFFICIENT {
            self.lower_level()
        } else if distance <= radius {
            self.lower_level().lower_level()
        } else {
            BLACK_SIGNAL_QUALITY
        }
    }
    
    fn lower_level(self) -> Self {
        match self.level {
            SignalLevel::Black | SignalLevel::Red => BLACK_SIGNAL_QUALITY,
            SignalLevel::Yellow                   => RED_SIGNAL_QUALITY,
            SignalLevel::Green                    => YELLOW_SIGNAL_QUALITY,
        }
    }

    #[must_use]
    pub fn area_radius_on(&self, frequency: Megahertz) -> Meter {
        self.strength.area_radius_on(frequency)
    }

    #[must_use]
    pub fn is_black(&self) -> bool {
        matches!(self.level, SignalLevel::Black)
    }
    
    #[must_use]
    pub fn is_red(&self) -> bool {
        matches!(self.level, SignalLevel::Red)
    }
    
    #[must_use]
    pub fn is_yellow(&self) -> bool {
        matches!(self.level, SignalLevel::Yellow)
    }

    #[must_use]
    pub fn is_green(&self) -> bool {
        matches!(self.level, SignalLevel::Green)
    }
}

impl From<SignalStrength> for SignalQuality {
    fn from(strength: SignalStrength) -> Self {
        let level = if strength > MAX_YELLOW_SIGNAL_STRENGTH {
            SignalLevel::Green
        } else if strength > MAX_RED_SIGNAL_STRENGTH {
            SignalLevel::Yellow
        } else if strength > MAX_BLACK_SIGNAL_STRENGTH {
            SignalLevel::Red
        } else {
            SignalLevel::Black
        };

        Self { strength, level }
    }
}

impl From<StrengthValue> for SignalQuality {
    fn from(value: StrengthValue) -> Self {
        Self::from(SignalStrength::new(value))
    }
}

impl PartialOrd for SignalQuality {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.strength.partial_cmp(&other.strength)
    }
}
