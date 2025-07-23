use std::path::PathBuf;

use super::config::GeneralConfig;


pub use premade::DEVICE_MAX_POWER;


use custom::custom;
use premade::{
    gps_only, gps_spoofing, malware_infection, movement, signal_loss_response
};


mod custom;
mod premade;


#[derive(Clone)]
pub enum Example {
    Custom(PathBuf),
    GPSEWD,
    GPSSpoofing,
    MalwareInfection,
    Movement,
    SignalLossResponse,
}

impl Example {
    pub fn execute(&self, general_config: &GeneralConfig) {
        if let Self::Custom(model_path) = self {
            custom(
                model_path, 
                general_config.model_player_config(), 
            );
            return;
        } 

        let example_function = self.premade_example_function();

        example_function(general_config);
    }

    fn premade_example_function(&self) -> fn(&GeneralConfig) {
        match self {
            Self::Custom(_)          => panic!("Custom function not allowed"),
            Self::GPSEWD             => gps_only,
            Self::GPSSpoofing        => gps_spoofing,
            Self::MalwareInfection   => malware_infection,
            Self::Movement           => movement,
            Self::SignalLossResponse => signal_loss_response,
        }
    }
}
