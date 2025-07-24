use serde::{Deserialize, Serialize};

use crate::backend::device::{Device, IdToDelayMap, IdToDeviceMap};
use crate::backend::mathphysics::{delay_to, Frequency, Millisecond, Position};
use crate::backend::signal::{Data, SignalQueue};


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GPS(Device);

impl GPS {
    #[must_use]
    pub fn new(device: Device) -> Self {
        Self(device)
    }
    
    #[must_use]
    pub fn device(&self) -> &Device {
        &self.0
    }
    
    #[must_use]
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.0
    }

    pub fn add_gps_signals_to_queue(
        &self,
        signal_queue: &mut SignalQueue,
        device_map: &IdToDeviceMap,
        current_time: Millisecond,
        delay_multiplier: f32,
    ) {
        for device in device_map.devices() {
            let Ok(gps_signal) = self.0.create_signal_for(
                device,
                Some(Data::GPS(*device.position())), 
                Frequency::GPS
            ) else {
                continue;
            };

            let delay = delay_to(
                self.0.distance_to(device), 
                delay_multiplier
            );
            
            signal_queue.add_entry(
                current_time, 
                gps_signal,
                IdToDelayMap::from([(device.id(), delay)])
            );
        }    
    }
}
