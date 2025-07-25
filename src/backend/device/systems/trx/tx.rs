use serde::{Deserialize, Serialize};

use crate::backend::mathphysics::{Frequency, Megahertz, Meter};
use crate::backend::signal::{FreqToQualityMap, SignalQuality};


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum TXModuleType {
    Level,
    #[default]
    Strength,
}


// By default we create a non-functioning `TXModule` based on signal strength.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TXModule {
    module_type: TXModuleType, 
    signal_quality_map: FreqToQualityMap
}

impl TXModule {
    #[must_use]
    pub fn new(
        module_type: TXModuleType, 
        signal_quality_map: FreqToQualityMap
    ) -> Self {
        Self { module_type, signal_quality_map }
    }

    #[must_use]
    pub fn signal_quality_map(&self) -> &FreqToQualityMap {
        &self.signal_quality_map
    }

    #[must_use]
    pub fn signal_quality_on(
        &self, 
        frequency: &Frequency
    ) -> Option<&SignalQuality> {
        self.signal_quality_map.get(frequency)
    }
    
    #[must_use]
    pub fn signal_quality_at(
        &self, 
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalQuality> {
        match self.module_type {
            TXModuleType::Level    => 
                self.signal_quality_at_by_level(distance, frequency),
            TXModuleType::Strength => 
                self.signal_quality_at_by_strength(distance, frequency),
        }
    }
    
    fn signal_quality_at_by_level(
        &self,
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalQuality> {
        self
            .signal_quality_on(&frequency)
            .map(|signal_quality| 
                signal_quality.at_by_level(frequency as Megahertz, distance)
            )
    }
    
    fn signal_quality_at_by_strength(
        &self,
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalQuality> {
        self
            .signal_quality_on(&frequency)
            .map(|signal_quality| 
                signal_quality.at_by_strength(frequency as Megahertz, distance)
            )
    }
}
