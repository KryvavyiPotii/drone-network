use serde::{Deserialize, Serialize};

use crate::backend::mathphysics::{Frequency, Megahertz, Meter};
use crate::backend::signal::{BLACK_SIGNAL_LEVEL, FreqToLevelMap, SignalLevel};


// By default we create a non-functioning TXModule.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TXModule {
    signal_levels: FreqToLevelMap
}

impl TXModule {
    #[must_use]
    pub fn new(signal_levels: FreqToLevelMap) -> Self {
        Self { signal_levels }
    }

    #[must_use]
    pub fn signal_levels(&self) -> &FreqToLevelMap {
        &self.signal_levels
    }

    #[must_use]
    pub fn signal_level_on(&self, frequency: &Frequency) -> &SignalLevel {
        self.signal_levels
            .get(frequency)
            .unwrap_or(&BLACK_SIGNAL_LEVEL)
    }
    
    #[must_use]
    pub fn signal_level_at_by_color(
        &self,
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalLevel> {
        let signal_level = self
            .signal_level_on(&frequency)
            .at_by_color(frequency as Megahertz, distance);
        
        if signal_level.is_black() {
            return None;
        } 

        Some(signal_level)
    }
    
    #[must_use]
    pub fn signal_level_at_by_strength(
        &self,
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalLevel> {
        let signal_level = self
            .signal_level_on(&frequency)
            .at(frequency as Megahertz, distance);
        
        if signal_level.is_black() {
            return None;
        } 

        Some(signal_level)
    }
}
