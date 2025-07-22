use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::mathphysics::{Megahertz, Meter, Millisecond};
use crate::backend::signal::{
    FreqToLevelMap, Signal, SignalArea, SignalLevel
};

pub use rx::{ReceivedSignal, RXError, RXModule};
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


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum TRXSystemType {
    Color,
    #[default]
    Strength,
}


#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TRXSystem {
    trx_system_type: TRXSystemType,
    tx_module: TXModule, 
    rx_module: RXModule,
}

impl TRXSystem {
    #[must_use]
    pub fn new(
        trx_system_type: TRXSystemType,
        tx_module: TXModule, 
        rx_module: RXModule,
    ) -> Self {
        Self {
            trx_system_type,
            tx_module,
            rx_module
        }
    }

    #[must_use]
    pub fn tx_signal_levels(&self) -> &FreqToLevelMap {
        self.tx_module.signal_levels() 
    }

    #[must_use]
    pub fn tx_signal_level_on(&self, frequency: &Megahertz) -> &SignalLevel {
        self.tx_module.signal_level_on(frequency) 
    }

    #[must_use]
    pub fn area_on(&self, frequency: Megahertz) -> SignalArea {
        SignalArea::from_level(
            *self.tx_module.signal_level_on(&frequency),
            frequency
        )
    }

    #[must_use]
    pub fn tx_signal_level_at(
        &self, 
        distance: Meter,
        frequency: Megahertz,
    ) -> Option<SignalLevel> {
        match self.trx_system_type {
            TRXSystemType::Color    => 
                self.tx_module.signal_level_at_by_color(distance, frequency),
            TRXSystemType::Strength => 
                self.tx_module.signal_level_at_by_strength(distance, frequency),
        }
    }
    
    #[must_use]
    pub fn transmits_at(
        &self, 
        distance: Meter, 
        frequency: Megahertz
    ) -> bool {
        self.tx_signal_level_at(distance, frequency).is_some()
    }
   
    #[must_use]
    pub fn receives_signal_on(&self, frequency: &Megahertz) -> bool {
        self.rx_module.receives_signal_on(frequency)
    }

    #[must_use]
    pub fn received_signals(&self) -> Vec<ReceivedSignal> {
        self.rx_module.received_signals()
    }
    
    #[must_use]
    pub fn received_signal_on(
        &self, 
        frequency: &Megahertz
    ) -> Option<&ReceivedSignal> {
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
