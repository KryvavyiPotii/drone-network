use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::device::DeviceId;
use super::malware::Malware;
use super::mathphysics::{Frequency, Point3D};
use super::task::Task;


pub use strength::*;
pub use queue::*;


pub mod strength;
pub mod queue;


pub type FreqToStrengthMap = HashMap<Frequency, SignalStrength>;


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Data {
    GPS(Point3D),
    Malware(Malware),
    SetTask(Task),
    Noise,
}


// Using `source_id` and `destination_id` is not realistic for signal but it is
// required for device communication to function. 
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    source_id: DeviceId,
    destination_id: DeviceId,
    data: Data,
    frequency: Frequency,
    strength: SignalStrength,
}

impl Signal {
    #[must_use]
    pub fn new(
        source_id: DeviceId,
        destination_id: DeviceId,
        data: Data,
        frequency: Frequency,
        strength: SignalStrength,
    ) -> Self {
        Self { 
            source_id,
            destination_id,
            data,
            frequency,
            strength, 
        }
    }

    #[must_use]
    pub fn to_noise(&self) -> Self {
        Self { data: Data::Noise, ..*self }
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
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[must_use]
    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    #[must_use]
    pub fn strength(&self) -> &SignalStrength {
        &self.strength
    }
    
    #[must_use]
    pub fn malware(&self) -> Option<&Malware> {
        if let Data::Malware(malware) = &self.data {
            return Some(malware);
        }

        None
    }
    
    #[must_use]
    pub fn task(&self) -> Option<&Task> {
        if let Data::SetTask(task) = &self.data {
            return Some(task);
        }

        None
    }

    #[must_use]
    pub fn is_malware(&self) -> bool {
        matches!(self.data, Data::Malware(_))
    }
}
