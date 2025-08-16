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
    EWD { 
        ew_frequency: Frequency, 
        ewd_area_radius: Meter
    },
    GPSSpoofing {
        spoofer_area_radius: Meter
    },
    MalwareInfection {
        malware: Malware, 
        attacker_area_radius: Meter, 
    },
    Movement,
    SignalLossResponse,
}

impl Example {
    pub fn execute(&self, general_config: &GeneralConfig) {
        match self {
            Self::Custom(json_path)                                   => 
                custom(json_path, general_config.model_player_config()),
            Self::EWD { ew_frequency, ewd_area_radius }               => 
                ewd(general_config, *ew_frequency, *ewd_area_radius),
            Self::GPSSpoofing { spoofer_area_radius }                 => 
                gps_spoofing(general_config, *spoofer_area_radius),
            Self::MalwareInfection { malware, attacker_area_radius, } => 
                malware_infection(
                    general_config, 
                    *malware,
                    *attacker_area_radius,
                ),
            Self::Movement           => movement(general_config),
            Self::SignalLossResponse => signal_loss_response(general_config),
        }
    }
}
