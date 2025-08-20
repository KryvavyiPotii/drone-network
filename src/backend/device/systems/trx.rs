use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::mathphysics::{Frequency, Megahertz, Meter, Millisecond};
use crate::backend::signal::{FreqToStrengthMap, Signal, SignalStrength};

pub use rx::{SignalRecord, RXError, RXModule};
pub use tx::TXModule;


mod rx;
mod tx;


#[derive(Error, Debug)]
pub enum TRXSystemError {
    #[error("RX module failed with error `{0}`")]
    RXModuleError(#[from] RXError),
    #[error("Receiver can not be reached")]
    RXOutOfRange,
    #[error("Signal destination ID does not match rx-device ID")]
    WrongSignalDestination,
    #[error("Signal source ID does not match tx-device ID")]
    WrongSignalSource,
}


#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TRXSystem {
    tx_module: TXModule, 
    rx_module: RXModule,
}

impl TRXSystem {
    #[must_use]
    pub fn new(tx_module: TXModule, rx_module: RXModule) -> Self {
        Self { tx_module, rx_module }
    }

    #[must_use]
    pub fn tx_signal_strength_map(&self) -> &FreqToStrengthMap {
        self.tx_module.signal_strength_map() 
    }

    #[must_use]
    pub fn tx_signal_strength_on(
        &self, 
        frequency: &Frequency
    ) -> Option<&SignalStrength> {
        self.tx_module.signal_strength_on(frequency) 
    }

    #[must_use]
    pub fn area_radius_on(&self, frequency: Frequency) -> Meter {
        self.tx_module
            .signal_strength_on(&frequency)
            .map_or(
                0.0, 
                |tx_signal_strength| 
                    tx_signal_strength.area_radius_on(frequency as Megahertz)
            )
    }

    #[must_use]
    pub fn tx_signal_strength_at(
        &self, 
        distance: Meter,
        frequency: Frequency,
    ) -> Option<SignalStrength> {
        self.tx_module.signal_strength_at(distance, frequency)
    }
    
    #[must_use]
    pub fn transmits_at(
        &self, 
        distance: Meter, 
        frequency: Frequency
    ) -> bool {
        self.tx_module.signal_strength_at(distance, frequency).is_some()
    }
   
    #[must_use]
    pub fn receives_signal_on(&self, frequency: &Frequency) -> bool {
        self.rx_module.receives_signal_on(frequency)
    }

    #[must_use]
    pub fn received_signals(&self) -> Vec<SignalRecord> {
        self.rx_module.received_signals()
    }
    
    #[must_use]
    pub fn received_signal_on(
        &self, 
        frequency: &Frequency
    ) -> Option<&SignalRecord> {
        self.rx_module.received_signal_on(frequency)
    }
     
    /// # Errors
    ///
    /// Will return `Err` if the RX module fails.
    pub fn receive_signal(
        &mut self,
        signal: Signal,
        time: Millisecond
    ) -> Result<(), TRXSystemError> {
        self.rx_module.receive_signal(signal, time)?;

        Ok(())
    }

    pub fn clear_received_signals(&mut self) {
        self.rx_module.clear_signals();
    }
}
