use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::device::systems::TRXSystemError;
use crate::backend::device::{Device, IdToDelayMap};
use crate::backend::malware::Malware;
use crate::backend::mathphysics::{
    delay_to, Frequency, Millisecond, Point3D, Position
};
use crate::backend::signal::{Data, Signal, SignalQueue};


#[derive(Error, Debug)]
pub enum AttackError {
    #[error("Target device is out of attacker device reach")]
    TargetOutOfRange,
    #[error("TRX system failed with error `{0}`")]
    TRXSystemError(#[from] TRXSystemError),
}


pub fn add_malware_signals_to_queue(
    source_device: &Device,
    destination_device: &Device,
    malware_list: &[Malware],
    signal_queue: &mut SignalQueue,
    current_time: Millisecond,
    delay_multiplier: f32,
) {
    let Some(signal_quality) = source_device.tx_signal_quality_at(
        destination_device, 
        Frequency::Control
    ) else {
        return;
    };

    if signal_quality.is_black() {
        return;
    }
    
    let delay = delay_to(
        source_device.distance_to(destination_device), 
        delay_multiplier
    );
    let delay_map = IdToDelayMap::from([(destination_device.id(), delay)]);


    for malware in malware_list {
        let Some(malware_spread_delay) = malware.spread_delay() else {
            continue;
        };

        let malware_signal = Signal::new(
            source_device.id(),
            destination_device.id(),
            Data::Malware(*malware), 
            Frequency::Control, 
            signal_quality
        );

        signal_queue.add_entry(
            current_time + malware_spread_delay, 
            malware_signal, 
            delay_map.clone()
        );
    }
}


#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AttackType {
    ElectronicWarfare,
    GPSSpoofing(Point3D),
    MalwareDistribution(Malware)
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackerDevice {
    device: Device,
    attack_type: AttackType
}

impl AttackerDevice {
    #[must_use]
    pub fn new(device: Device, attack_type: AttackType) -> Self {
        Self { device, attack_type }
    }

    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }

    #[must_use]
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    #[must_use]
    pub fn attack_type(&self) -> AttackType {
        self.attack_type
    }

    /// # Errors
    ///
    /// Will return `Err` if target device is out of attacker's range or 
    /// attacker's TRX system fails. 
    pub fn execute_attack(
        &self,
        target_device: &Device,
        signal_queue: &mut SignalQueue,
        current_time: Millisecond,
        delay_multiplier: f32,
    ) -> Result<(), AttackError> {
        let signals_to_send = self.generate_signals(target_device)?;

        let delay = delay_to(
            self.device.distance_to(target_device), 
            delay_multiplier
        );
        let delay_map = IdToDelayMap::from([(target_device.id(), delay)]);

        for signal in &signals_to_send {
            signal_queue.add_entry(current_time, *signal, delay_map.clone());
        };

        Ok(())
    }

    fn generate_signals(
        &self, 
        target_device: &Device
    ) -> Result<Vec<Signal>, AttackError> {
        match self.attack_type {
            AttackType::ElectronicWarfare             => 
                self.generate_noise_on_all_frequencies(target_device),
            AttackType::GPSSpoofing(spoofed_position) => {
                let spoofing_signal = self.generate_gps_spoofing_signal(
                    target_device, 
                    spoofed_position,
                )?;

                Ok(vec![spoofing_signal])
            },
            AttackType::MalwareDistribution(malware)  => {
                let malware_signal = self.generate_signal_with_malware(
                    target_device, 
                    malware,
                )?;

                Ok(vec![malware_signal])
            },
        }
    }
    
    fn generate_noise_on_all_frequencies(
        &self,
        target_device: &Device,
    ) -> Result<Vec<Signal>, AttackError> {
        let signals_to_send: Vec<Signal> = self.device
            .tx_signal_quality_map()
            .keys() 
            .filter_map(|frequency| {
                self.device.create_signal_for(
                    target_device, 
                    Data::Noise, 
                    *frequency
                ).ok()
            })
            .collect();

        if signals_to_send.is_empty() {
            return Err(AttackError::TargetOutOfRange);
        }
        
        Ok(signals_to_send)
    }

    fn generate_gps_spoofing_signal(
        &self,
        target_device: &Device,
        spoofed_position: Point3D,
    ) -> Result<Signal, AttackError> {
        self.device.create_signal_for(
            target_device, 
            Data::GPS(spoofed_position), 
            Frequency::GPS,
        ).map_err(|_| AttackError::TargetOutOfRange)
    }
    
    fn generate_signal_with_malware(
        &self,
        target_device: &Device,
        malware: Malware,
    ) -> Result<Signal, AttackError> {
        self.device.create_signal_for(
            target_device, 
            Data::Malware(malware), 
            Frequency::Control
        ).map_err(|_| AttackError::TargetOutOfRange)
    }
}
