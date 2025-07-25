use derive_more::{Add, Div, Mul, Sub, Display};
use serde::{Deserialize, Serialize};

use crate::backend::mathphysics::{wave_length_in_meters, Megahertz, Meter};


pub const GREEN_SIGNAL_STRENGTH_VALUE: StrengthValue = 100.0;

pub const MAX_BLACK_SIGNAL_STRENGTH: SignalStrength  = SignalStrength(1.0);
pub const MAX_RED_SIGNAL_STRENGTH: SignalStrength    = SignalStrength(
    GREEN_SIGNAL_STRENGTH_VALUE * 0.2
);
pub const MAX_YELLOW_SIGNAL_STRENGTH: SignalStrength = SignalStrength(
    GREEN_SIGNAL_STRENGTH_VALUE * 0.5
);
pub const GREEN_SIGNAL_STRENGTH: SignalStrength      = SignalStrength(
    GREEN_SIGNAL_STRENGTH_VALUE
);


// Const for proper signal strength scaling at distance.
const SIGNAL_STRENGTH_SCALING: StrengthValue = 2_500.0; 


pub type StrengthValue = f32;


#[derive(
    Clone, Copy, Debug, Display, Default, Add, Sub, Mul, Div, PartialEq, 
    PartialOrd, Serialize, Deserialize
)]
#[display("{_0}")]
pub struct SignalStrength(StrengthValue);

impl SignalStrength {
    #[must_use]
    pub fn new(value: StrengthValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn value(&self) -> StrengthValue {
        self.0
    }
    
    #[must_use]
    pub fn from_area_radius(area_radius: Meter, frequency: Megahertz) -> Self {
        let wave_length = wave_length_in_meters(frequency);

        // TX signal strength is such signal strength that grants at least
        // black RX signal strength on the signal area radius.
        // So, the actual formula is:
        //     tx_signal_strength = (
        //         MAX_BLACK_SIGNAL_STRENGTH * radius / wave_length
        //     ).powi()
        // We do not use multiplication by MAX_BLACK_SIGNAL_STRENGTH because it 
        // is equal to 1.0.
        let tx_strength_value = (
            area_radius / wave_length
        ).powi(2) / SIGNAL_STRENGTH_SCALING;

        Self(tx_strength_value)
    }
    
    #[must_use]
    pub fn at(&self, frequency: Megahertz, distance: Meter) -> Self {
        if *self <= MAX_BLACK_SIGNAL_STRENGTH {
            return Self::default();
        }

        let wave_length = wave_length_in_meters(frequency);

        // For now we ignore division by distance, if it is less than a wave
        // length. However, in the future free-space path loss model may 
        // changed for this particular case.
        let signal_strength_at = if distance <= wave_length {
            wave_length.powi(2)
        } else {
            (wave_length / distance).powi(2)
        } * self.0 * SIGNAL_STRENGTH_SCALING; 

        Self(signal_strength_at)
    }
    
    #[must_use]
    pub fn area_radius_on(&self, frequency: Megahertz) -> Meter {
        if *self <= MAX_BLACK_SIGNAL_STRENGTH {
            return 0.0;
        }
       
        let wave_length = wave_length_in_meters(frequency);

        // The area radius is a minimal distance from the tx at which 
        // the signal level is black.
        // So, the actual formula is:
        //     radius = wave_length * (
        //         tx_signal_strength / MAX_BLACK_SIGNAL_STRENGTH
        //     ).sqrt()
        // We do not use division by MAX_BLACK_SIGNAL_STRENGTH because it 
        // is equal to 1.0.
        wave_length * (self.0 * SIGNAL_STRENGTH_SCALING).sqrt() 
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn somewhat_realistic_area_radius() {
        let tx_signal_strength = GREEN_SIGNAL_STRENGTH;
        let frequency = 5_000;
        let distance_outside_tx_area = 40.0;
        let distance_far_from_tx     = 15.0;
        let distance_close_to_tx     = 5.0;
        let distance_next_to_tx      = 3.0;
        
        let black_signal_strength = tx_signal_strength.at(
            frequency, 
            distance_outside_tx_area
        );

        assert!(black_signal_strength <= MAX_BLACK_SIGNAL_STRENGTH);
        
        let red_signal_strength = tx_signal_strength.at(
            frequency, 
            distance_far_from_tx
        );

        assert!(
            red_signal_strength > MAX_BLACK_SIGNAL_STRENGTH
            && red_signal_strength <= MAX_RED_SIGNAL_STRENGTH
        );
        
        let yellow_signal_strength = tx_signal_strength.at(
            frequency, 
            distance_close_to_tx
        );
        
        assert!(
            yellow_signal_strength > MAX_RED_SIGNAL_STRENGTH
            && yellow_signal_strength <= MAX_YELLOW_SIGNAL_STRENGTH
        );

        let green_signal_strength = tx_signal_strength.at(
            frequency,
            distance_next_to_tx
        );

        assert!(green_signal_strength > MAX_YELLOW_SIGNAL_STRENGTH);
    }
}
