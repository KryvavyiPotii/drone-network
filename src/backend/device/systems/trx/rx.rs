use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::mathphysics::{Frequency, Millisecond};
use crate::backend::signal::{Data, FreqToQualityMap, Signal, SignalQuality};


// The first element - time at which a signal was received.
// The second element - the signal.
pub type SignalRecord = (Millisecond, Signal);


const RECEIVE_GREEN_SIGNAL: f64  = 0.95;
const RECEIVE_YELLOW_SIGNAL: f64 = 0.75;
const RECEIVE_RED_SIGNAL: f64    = 0.5;
const RECEIVE_BLACK_SIGNAL: f64  = 0.1;


fn signal_reached_rx(signal_quality: SignalQuality) -> bool {
    rand::random_bool(
        signal_reach_rx_probability(signal_quality)
    )
}

fn signal_reach_rx_probability(signal_quality: SignalQuality) -> f64 {
    if signal_quality.is_green() {
        RECEIVE_GREEN_SIGNAL
    } else if signal_quality.is_yellow() {
        RECEIVE_YELLOW_SIGNAL
    } else if signal_quality.is_red() {
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
    max_signal_quality_map: FreqToQualityMap,
    received_signals: Vec<SignalRecord>,
}

impl RXModule {
    #[must_use]
    pub fn new(max_signal_quality_map: FreqToQualityMap) -> Self {
        Self { 
            max_signal_quality_map,
            received_signals: Vec::new() 
        }
    }

    #[must_use]
    pub fn receives_signal_on(&self, frequency: &Frequency) -> bool {
        self.received_signals
            .iter()
            .any(|(_, signal)| 
                signal.frequency() == *frequency 
                    && !matches!(signal.data(), Data::Noise)
            )
    }

    #[must_use]
    pub fn received_signals(&self) -> Vec<SignalRecord> {
        self.received_signals.clone()
    }

    #[must_use]
    pub fn received_signal_on(
        &self, 
        frequency: &Frequency, 
    ) -> Option<&SignalRecord> {
        self.received_signals
            .iter()
            .find(|received_signal| received_signal.1.frequency() == *frequency)
    }
    
    /// # Errors
    ///
    /// Will return `Err` if RX module does not listen on received signal's 
    /// frequency, received signal's quality is lower than current signal's or 
    /// it is higher than maximum signal quality on respective frequency.
    pub fn receive_signal(
        &mut self, 
        signal: Signal,
        time: Millisecond
    ) -> Result<(), RXError> {
        if !signal_reached_rx(*signal.quality()) {
            return Err(RXError::SignalNotReceived);
        }

        let max_signal_quality = *self.max_signal_quality_on(
            signal.frequency()
        )?;

        if let Some((_, current_signal)) = self.received_signal_on(
            &signal.frequency()
        ) {
            if current_signal.quality() > signal.quality() {
                return Err(RXError::SignalTooWeak);
            }
        }

        self.remove_current_received_signal_on(signal.frequency());

        if *signal.quality() > max_signal_quality {
            self.received_signals.push((time, signal.to_noise()));

            return Err(RXError::NoiseReceived);
        }

        self.received_signals.push((time, signal));
        
        Ok(())
    }

    fn max_signal_quality_on(
        &self, 
        frequency: Frequency, 
    ) -> Result<&SignalQuality, RXError> {
        let Some(max_signal_quality) = self.max_signal_quality_map.get(
            &frequency
        ) else {
            return Err(RXError::NotListeningOnFrequency);
        };

        Ok(max_signal_quality)
    }

    fn remove_current_received_signal_on(&mut self, frequency: Frequency) {
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
