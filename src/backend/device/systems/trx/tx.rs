use serde::{Deserialize, Serialize};

use crate::backend::mathphysics::{Frequency, Megahertz, Meter};
use crate::backend::signal::{FreqToStrengthMap, SignalStrength};


// By default we create a non-functioning `TXModule` based on signal strength.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TXModule {
    signal_strength_map: FreqToStrengthMap
}

impl TXModule {
    #[must_use]
    pub fn new(
        signal_strength_map: FreqToStrengthMap
    ) -> Self {
        Self { signal_strength_map }
    }

    #[must_use]
    pub fn signal_strength_map(&self) -> &FreqToStrengthMap {
        &self.signal_strength_map
    }

    #[must_use]
    pub fn signal_strength_on(
        &self, 
        frequency: &Frequency
    ) -> Option<&SignalStrength> {
        self.signal_strength_map.get(frequency)
    }
    
    #[must_use]
    pub fn signal_strength_at(
        &self,
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalStrength> {
        self
            .signal_strength_on(&frequency)
            .map(|signal_strength| 
                signal_strength.at(frequency as Megahertz, distance)
            )
    }
}
