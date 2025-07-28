use std::path::PathBuf;

use crate::backend::mathphysics::Frequency;

use super::config::GeneralConfig;


pub use premade::DEVICE_MAX_POWER;


use custom::custom;
use premade::{
    ewd, gps_spoofing, malware_infection, movement, signal_loss_response
};


mod custom;
mod premade;


#[derive(Clone)]
pub enum Example {
    Custom(PathBuf),
    EWD(Frequency),
    GPSSpoofing,
    MalwareInfection,
    Movement,
    SignalLossResponse,
}

impl Example {
    pub fn execute(&self, general_config: &GeneralConfig) {
        match self {
            Self::Custom(json_path)  => 
                custom(json_path, general_config.model_player_config()),
            Self::EWD(frequency)     => ewd(general_config, *frequency),
            Self::GPSSpoofing        => gps_spoofing(general_config),
            Self::MalwareInfection   => malware_infection(general_config),
            Self::Movement           => movement(general_config),
            Self::SignalLossResponse => signal_loss_response(general_config),
        }
    }
}
