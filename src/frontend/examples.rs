use std::path::PathBuf;

use crate::backend::malware::Malware;
use crate::backend::mathphysics::Point3D;

use super::config::GeneralConfig;


pub use devsetup::DEVICE_MAX_POWER;


use custom::custom;
use devsetup::{
    NetworkPosition, generate_drone_positions, generate_drone_vulnerabilities, 
    NETWORK_ORIGIN
};
use premade::{
    gps_only, gps_spoofing, indicator_malware, malware_infection, movement, 
    signal_loss_response
};


mod custom;
mod devsetup;
mod premade;


type ExampleFn = fn(&GeneralConfig, &[Point3D], &[Vec<Malware>]);


fn default_network_position(network_origin: Point3D) -> NetworkPosition {
    NetworkPosition::new(
        network_origin,
        -40.0..40.0,
        -40.0..40.0,
        -20.0..20.0,
    )
}


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
                general_config.render_config()
            );
            return;
        } 

        self.execute_premade(general_config);
    }

    fn execute_premade(&self, general_config: &GeneralConfig) {
        let network_position = default_network_position(self.network_origin());
        
        let drone_positions  = generate_drone_positions(
            general_config.model_config().drone_count(), 
            &network_position
        );
        let malware_list     = general_config.malware_list(indicator_malware());
        let vulnerabilities  = generate_drone_vulnerabilities(
            general_config.model_config().drone_count(), 
            &malware_list
        );
        
        let example_function = self.premade_example_function();

        example_function(
            general_config, 
            &drone_positions, 
            &vulnerabilities
        );
    }

    fn network_origin(&self) -> Point3D {
        match self {
            Example::MalwareInfection => Point3D::new(50.0, 50.0, 0.0),
            _                         => NETWORK_ORIGIN,
        }
    }

    fn premade_example_function(&self) -> ExampleFn {
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
