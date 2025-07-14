use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::device::DeviceId;
use super::malware::Malware;
use super::mathphysics::{Megahertz, Point3D};
use super::task::Task;


pub use area::*;
pub use level::*;
pub use strength::*;
pub use queue::*;


pub mod area;
pub mod level;
pub mod strength;
pub mod queue;


pub const GPS_L1_FREQUENCY: Megahertz      = 1_575;
pub const WIFI_2_4GHZ_FREQUENCY: Megahertz = 2_400;

// Const for proper signal strength scaling at distance.
const SIGNAL_STRENGTH_SCALING: f32 = 2_500.0; 


pub type FreqToLevelMap = HashMap<Megahertz, SignalLevel>;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Data {
    GPS(Point3D),
    Malware(Malware),
    SetTask(Task),
}


// Using `source_id` and `destination_id` is not realistic for signal but it is
// required for device communication to function. 
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    source_id: DeviceId,
    destination_id: DeviceId,
    data: Option<Data>,
    frequency: Megahertz,
    level: SignalLevel,
}

impl Signal {
    #[must_use]
    pub fn new(
        source_id: DeviceId,
        destination_id: DeviceId,
        data: Option<Data>,
        frequency: Megahertz,
        level: SignalLevel,
    ) -> Self {
        Self { 
            source_id,
            destination_id,
            data,
            frequency,
            level, 
        }
    }

    #[must_use]
    pub fn to_noise(&self) -> Self {
        Self { data: None, ..*self }
    }
    
    #[must_use]
    pub fn source_id(&self) -> DeviceId {
        self.source_id
    }
    
    #[must_use]
    pub fn destination_id(&self) -> DeviceId {
        self.destination_id
    }

    #[must_use]
    pub fn data(&self) -> Option<&Data> {
        self.data.as_ref()
    }

    #[must_use]
    pub fn frequency(&self) -> Megahertz {
        self.frequency
    }

    #[must_use]
    pub fn level(&self) -> &SignalLevel {
        &self.level
    }
    
    #[must_use]
    pub fn malware(&self) -> Option<&Malware> {
        if let Some(Data::Malware(malware)) = &self.data {
            return Some(malware);
        }

        None
    }
    
    #[must_use]
    pub fn task(&self) -> Option<&Task> {
        if let Some(Data::SetTask(task)) = &self.data {
            return Some(task);
        }

        None
    }

    #[must_use]
    pub fn is_malware(&self) -> bool {
        matches!(self.data, Some(Data::Malware(_)))
    }
}
