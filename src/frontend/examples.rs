use std::path::PathBuf;

use crate::backend::malware::Malware;
use crate::backend::mathphysics::{Frequency, Meter};

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
    EWD(Frequency, Meter),
    GPSSpoofing(Meter),
    MalwareInfection(Malware, Meter, bool),
    Movement,
    SignalLossResponse,
}

impl Example {
    pub fn execute(&self, general_config: &GeneralConfig) {
        match self {
            Self::Custom(json_path)                                           => 
                custom(json_path, general_config.model_player_config()),
            Self::EWD(frequency, ewd_area_radius)                             => 
                ewd(general_config, *frequency, *ewd_area_radius),
            Self::GPSSpoofing(spoofer_area_radius)                            => 
                gps_spoofing(general_config, *spoofer_area_radius),
            Self::MalwareInfection(
                malware, 
                attacker_area_radius, 
                display_propagation
            ) => 
                malware_infection(
                    general_config, 
                    *malware,
                    *attacker_area_radius,
                    *display_propagation
                ),
            Self::Movement           => movement(general_config),
            Self::SignalLossResponse => signal_loss_response(general_config),
        }
    }
}
