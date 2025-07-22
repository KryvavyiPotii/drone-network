use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::mathphysics::{Megahertz, Millisecond};
use crate::backend::signal::{FreqToLevelMap, Signal, SignalLevel};


// The first element - time at which a signal was received.
// The second element - the signal.
pub type ReceivedSignal = (Millisecond, Signal);


const RECEIVE_GREEN_SIGNAL: f64  = 0.95;
const RECEIVE_YELLOW_SIGNAL: f64 = 0.70;
const RECEIVE_RED_SIGNAL: f64    = 0.50;
const RECEIVE_BLACK_SIGNAL: f64  = 0.10;


fn signal_is_received(signal_level: SignalLevel) -> bool {
    rand::random_bool(
        signal_receive_probability(signal_level)
    )
}

fn signal_receive_probability(signal_level: SignalLevel) -> f64 {
    if signal_level.is_green() {
        RECEIVE_GREEN_SIGNAL
    } else if signal_level.is_yellow() {
        RECEIVE_YELLOW_SIGNAL
    } else if signal_level.is_red() {
        RECEIVE_RED_SIGNAL
    } else {
        RECEIVE_BLACK_SIGNAL
    }
}


#[derive(Debug, Error)]
pub enum RXError {
    #[error("RX module does not listen on signal's frequency")]
    NotListeningOnFrequency,
    #[error("Received signal is too strong to decode its data")]
    NoiseReceived,
    #[error("Failed to receive signal")]
    SignalNotReceived,
    #[error("RX module has already received stronger signal")]
    SignalTooWeak,
}


// By default we create a non-functioning RXModule.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RXModule {
    max_signal_levels: FreqToLevelMap,
    received_signals: Vec<ReceivedSignal>,
}

impl RXModule {
    #[must_use]
    pub fn new(max_signal_levels: FreqToLevelMap) -> Self {
        Self { 
            max_signal_levels,
            received_signals: Vec::new() 
        }
    }

    #[must_use]
    pub fn receives_signal_on(&self, frequency: &Megahertz) -> bool {
        self.received_signals
            .iter()
            .any(|(_, signal)| 
                signal.frequency() == *frequency && signal.data().is_some() 
            )
    }

    #[must_use]
    pub fn received_signals(&self) -> Vec<ReceivedSignal> {
        self.received_signals.clone()
    }

    #[must_use]
    pub fn received_signal_on(
        &self, 
        frequency: &Megahertz, 
    ) -> Option<&ReceivedSignal> {
        self.received_signals
            .iter()
            .find(|received_signal| received_signal.1.frequency() == *frequency)
    }
    
    /// # Errors
    ///
    /// Will return `Err` if RX module does not listen on received signal's 
    /// frequency, received signal's level is lower than current signal's or it 
    /// is higher than maximum signal level on respective frequency.
    pub fn receive_signal(
        &mut self, 
        signal: Signal,
        time: Millisecond
    ) -> Result<(), RXError> {
        let max_signal_level = *self.max_signal_level_on(signal.frequency())?;

        if let Some((_, current_signal)) = self.received_signal_on(
            &signal.frequency()
        ) {
            if current_signal.level() > signal.level() {
                return Err(RXError::SignalTooWeak);
            }
        }

        if !signal_is_received(*signal.level()) {
            return Err(RXError::SignalNotReceived);
        }

        self.remove_current_received_signal_on(signal.frequency());

        // Signals which level is higher than RX module's max, are viewed as 
        // noise.
        if *signal.level() > max_signal_level {
            self.received_signals.push((time, signal.to_noise()));
            return Err(RXError::NoiseReceived);
        }

        self.received_signals.push((time, signal));
        
        Ok(())
    }

    fn max_signal_level_on(
        &self, 
        frequency: Megahertz
    ) -> Result<&SignalLevel, RXError> {
        let Some(max_signal_level) = self.max_signal_levels.get(
            &frequency
        ) else {
            return Err(RXError::NotListeningOnFrequency);
        };
        
        if max_signal_level.is_black() {
            return Err(RXError::NotListeningOnFrequency);
        }

        Ok(max_signal_level)
    }

    fn remove_current_received_signal_on(&mut self, frequency: Megahertz) {
        let Some(current_signal_index) = self.received_signals
            .iter()
            .position(|(_, current_signal)| 
                current_signal.frequency() == frequency
            )
        else {
            return;
        };

        self.received_signals.remove(current_signal_index);
    }
    
    pub fn clear_signals(&mut self) {
        self.received_signals.clear();
    }
}
