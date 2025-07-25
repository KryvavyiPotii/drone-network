use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::mathphysics::PowerUnit;


#[derive(Error, Debug)]
pub enum PowerSystemError {
    #[error("Provided power is greater than current power")]
    NoPowerLeft,
}

#[derive(Error, Debug)]
pub enum PowerSystemBuildError {
    #[error("Power is greater than max power")]
    PowerIsGreaterThanMax,
}


// By default the system can supply any power, because its maximum power is 0.0.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PowerSystem {
    max_power: PowerUnit,
    power: PowerUnit,
}

impl PowerSystem {
    /// # Errors
    ///
    /// Will return `Err` if provided power is higher than provided max power.
    pub fn build(
        max_power: PowerUnit, 
        power: PowerUnit
    ) -> Result<Self, PowerSystemBuildError> {
        if power > max_power {
            return Err(PowerSystemBuildError::PowerIsGreaterThanMax);
        }

        Ok(Self { max_power, power })
    }

    #[must_use]
    pub fn max_power(&self) -> PowerUnit {
        self.max_power
    }

    #[must_use]
    pub fn power(&self) -> PowerUnit {
        self.power
    }

    /// # Errors
    ///
    /// Will return `Err` if the system consume all power.
    pub fn consume_power(
        &mut self, 
        power_to_consume: PowerUnit
    ) -> Result<(), PowerSystemError> {
        self.power = self.power.saturating_sub(power_to_consume);

        if self.power == 0 {
            return Err(PowerSystemError::NoPowerLeft)
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn default_power_system_does_not_supply_power() {
        let default_power_system = PowerSystem::default();

        assert_eq!(default_power_system.max_power(), 0);
    }

    #[test]
    fn building_power_system_with_power_greater_than_max_is_impossible() {
        let max_power      = 50;
        let too_high_power = max_power * 2;

        let result = PowerSystem::build(max_power, too_high_power);

        assert!(
            matches!(result, Err(PowerSystemBuildError::PowerIsGreaterThanMax))
        );
    }

    #[test]
    fn error_on_consuming_all_power() {
        let max_power = 10;
        let power     = max_power;
        
        let mut power_system = PowerSystem::build(max_power, power)
            .unwrap_or_else(|error| panic!("{}", error));

        assert!(
            matches!(
                power_system.consume_power(power),
                Err(PowerSystemError::NoPowerLeft)
            )
        );
        assert_eq!(power_system.power, 0);
    }
}    
