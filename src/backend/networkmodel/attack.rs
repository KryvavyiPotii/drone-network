use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::device::systems::TRXSystemError;
use crate::backend::CONTROL_FREQUENCY;
use crate::backend::device::{Device, IdToDelayMap};
use crate::backend::malware::Malware;
use crate::backend::mathphysics::{delay_to, Millisecond, Point3D, Position};
use crate::backend::signal::{Data, Signal, SignalQueue, GPS_L1_FREQUENCY};


#[derive(Error, Debug)]
pub enum AttackError {
    #[error("Target device is out of attacker device reach")]
    TargetOutOfRange,
    #[error("TRX system failed with error `{0}`")]
    TRXSystemError(#[from] TRXSystemError),
    #[error("Attacker device does not execute this type of attack")]
    WrongAttackType,
}


pub fn add_malware_signals_to_queue(
    source_device: &Device,
    destination_device: &Device,
    malware_list: &[Malware],
    signal_queue: &mut SignalQueue,
    current_time: Millisecond,
    delay_multiplier: f32,
) {
    let Some(signal_level) = source_device.tx_signal_level_at(
        destination_device, 
        CONTROL_FREQUENCY
    ) else {
        return;
    };

    for malware in malware_list {
        let Some(malware_spread_delay) = malware.spread_delay() else {
            continue;
        };

        let malware_data = Data::Malware(*malware);
        let malware_signal = Signal::new(
            source_device.id(),
            destination_device.id(),
            Some(malware_data), 
            CONTROL_FREQUENCY, 
            signal_level
        );

        let delay = delay_to(
            source_device.distance_to(destination_device), 
            delay_multiplier
        );

        signal_queue.add_entry(
            current_time + malware_spread_delay, 
            malware_signal, 
            IdToDelayMap::from([(destination_device.id(), delay)])
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
    pub fn attack_type(&self) -> AttackType {
        self.attack_type
    }

    /// # Errors
    ///
    /// Will return `Err` if incorrect attack method is called, target device is
    /// out of attacker's range or attacker's TRX system fails. 
    pub fn execute_attack(
        &self,
        target_device: &Device,
        signal_queue: &mut SignalQueue,
        current_time: Millisecond,
        delay_multiplier: f32,
    ) -> Result<(), AttackError> {
        match self.attack_type {
            AttackType::ElectronicWarfare      => 
                self.execute_electronic_warfare(
                    target_device, 
                    signal_queue, 
                    current_time,
                    delay_multiplier
                ),
            AttackType::GPSSpoofing(_)         => 
                self.spoof_gps(
                    target_device, 
                    signal_queue, 
                    current_time,
                    delay_multiplier
                ),
            AttackType::MalwareDistribution(_) =>
                self.spread_malware(
                    target_device, 
                    signal_queue, 
                    current_time,
                    delay_multiplier
                ),
        }
    }
    
    fn execute_electronic_warfare(
        &self, 
        target_device: &Device,
        signal_queue: &mut SignalQueue,
        current_time: Millisecond,
        delay_multiplier: f32,
    ) -> Result<(), AttackError> {
        let AttackType::ElectronicWarfare = self.attack_type else {
            return Err(AttackError::WrongAttackType);
        };
        
        let mut result = Err(AttackError::TargetOutOfRange);
        let distance = self.device.distance_to(target_device);

        for frequency in self.device.tx_signal_levels().keys() {
            let Ok(jamming_signal) = self.device.create_signal_for(
                target_device, 
                None, 
                *frequency
            ) else {
                continue;
            };

            let delay = delay_to(distance, delay_multiplier);

            signal_queue.add_entry(
                current_time, 
                jamming_signal, 
                IdToDelayMap::from([(target_device.id(), delay)])
            );

            result = Ok(());
        }

        result
    }

    fn spoof_gps(
        &self,
        target_device: &Device,
        signal_queue: &mut SignalQueue,
        current_time: Millisecond,
        delay_multiplier: f32,
    ) -> Result<(), AttackError> {
        let AttackType::GPSSpoofing(spoofed_position) = self.attack_type else {
            return Err(AttackError::WrongAttackType);
        };

        let Ok(spoofing_signal) = self.device.create_signal_for(
            target_device, 
            Some(Data::GPS(spoofed_position)), 
            GPS_L1_FREQUENCY
        ) else {
            return Err(AttackError::TargetOutOfRange);
        };

        let delay = delay_to(
            self.device.distance_to(target_device), 
            delay_multiplier
        );

        signal_queue.add_entry(
            current_time, 
            spoofing_signal, 
            IdToDelayMap::from([(target_device.id(), delay)])
        );

        Ok(())
    }
    
    fn spread_malware(
        &self,
        target_device: &Device,
        signal_queue: &mut SignalQueue,
        current_time: Millisecond,
        delay_multiplier: f32,
    ) -> Result<(), AttackError> {
        let AttackType::MalwareDistribution(malware) = self.attack_type else {
            return Err(AttackError::WrongAttackType);
        };
        
        let Ok(malware_signal) = self.device.create_signal_for(
            target_device, 
            Some(Data::Malware(malware)), 
            CONTROL_FREQUENCY
        ) else {
            return Err(AttackError::TargetOutOfRange);
        };
        
        let delay = delay_to(
            self.device.distance_to(target_device), 
            delay_multiplier
        );

        signal_queue.add_entry(
            current_time, 
            malware_signal,
            IdToDelayMap::from([(target_device.id(), delay)])
        );

        Ok(())
    }
}
