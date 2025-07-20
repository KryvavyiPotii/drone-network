use std::path::{Path, PathBuf};

use clap::ArgMatches;

use crate::backend::connections::Topology;
use crate::backend::device::systems::TRXSystemType;
use crate::backend::malware::{Malware, MalwareType};
use crate::backend::mathphysics::Millisecond;
use crate::frontend::{MALWARE_INFECTION_DELAY, MALWARE_SPREAD_DELAY};
use crate::frontend::config::{
    GeneralConfig, ModelConfig, ModelPlayerConfig, RenderConfig
};
use crate::frontend::examples::{Example, DEVICE_MAX_POWER};
use crate::frontend::renderer::{
    Pixel, PlotResolution, DEFAULT_AXES_RANGE,
    DEFAULT_CAMERA_ANGLE, DEFAULT_DEVICE_COLORING
};


pub const ARG_DELAY_MULTIPLIER: &str = "delay multiplier";
pub const ARG_DISPLAY_MALWARE_PROPAGATION: &str = "display malware propagation";
pub const ARG_DRONE_COUNT: &str      = "drone count";
pub const ARG_EXPERIMENT_TITLE: &str = "experiment title";
pub const ARG_INPUT_MODEL: &str      = "network model path";
pub const ARG_MALWARE_TYPE: &str     = "malware type";
pub const ARG_NETWORK_TOPOLOGY: &str = "network topology";
pub const ARG_OUTPUT_DIR: &str       = "output directory path";
pub const ARG_PLOT_CAPTION: &str     = "plot caption";
pub const ARG_PLOT_HEIGHT: &str      = "plot height";
pub const ARG_PLOT_WIDTH: &str       = "plot width";
pub const ARG_SIM_TIME: &str         = "simulation time";
pub const ARG_TRX_SYSTEM: &str       = "trx system type";

pub const EXP_CUSTOM: &str            = "custom";
pub const EXP_GPS_ONLY: &str          = "gpsewd";
pub const EXP_GPS_SPOOFING: &str      = "gpsspoof";
pub const EXP_MALWARE_INFECTION: &str = "malware";
pub const EXP_MOVEMENT: &str          = "move";
pub const EXP_SIGNAL_LOSS: &str       = "signalloss";

pub const MAL_DOS: &str       = "dos";
pub const MAL_INDICATOR: &str = "indicator";

pub const TOPOLOGY_BOTH: &str = "both";
pub const TOPOLOGY_MESH: &str = "mesh";
pub const TOPOLOGY_STAR: &str = "star";

pub const TRX_BOTH: &str     = "both";
pub const TRX_COLOR: &str    = "color";
pub const TRX_STRENGTH: &str = "strength";

pub const DEFAULT_DELAY_MULTIPLIER: &str = "0.0";
pub const DEFAULT_DRONE_COUNT: &str      = "100";
pub const DEFAULT_PLOT_CAPTION: &str     = "";
pub const DEFAULT_PLOT_HEIGHT: &str      = "300";
pub const DEFAULT_PLOT_WIDTH: &str       = "400";
pub const DEFAULT_SIM_TIME: &str         = "15000";


pub fn handle_arguments(matches: &ArgMatches) {
    let Some(experiment_title) = matches.get_one::<String>(
        ARG_EXPERIMENT_TITLE
    ) else {
        return;
    };
 
    let example = match experiment_title.as_str() {
        EXP_CUSTOM            => {
            let Some(model_path) = input_model_path(matches) else {
                return;
            };
            
            Example::Custom(model_path)
        },
        EXP_GPS_ONLY          => Example::GPSEWD,
        EXP_GPS_SPOOFING      => Example::GPSSpoofing,
        EXP_MALWARE_INFECTION => Example::MalwareInfection, 
        EXP_MOVEMENT          => Example::Movement,
        EXP_SIGNAL_LOSS       => Example::SignalLossResponse,
        _                     => return
    };

    let model_config = match example {
        Example::Custom(_) => ModelConfig::default(),
        _                  => model_config(matches),
    };
    
    example.execute(
        &GeneralConfig::new(
            model_config,
            model_player_config(matches),
            render_config(matches)
        )
    );
}

fn model_config(matches: &ArgMatches) -> ModelConfig {
    ModelConfig::new(
        trx_system_type(matches), 
        topology(matches),
        drone_count(matches),
        delay_multiplier(matches),
        malware(matches),
    )
}

fn model_player_config(matches: &ArgMatches) -> ModelPlayerConfig {
    ModelPlayerConfig::new(
        output_directory(matches), 
        simulation_time(matches),
    )
}

fn render_config(matches: &ArgMatches) -> RenderConfig {
    RenderConfig::new(
        plot_caption(matches), 
        plot_resolution(matches), 
        DEFAULT_AXES_RANGE,
        DEFAULT_CAMERA_ANGLE,
        DEFAULT_DEVICE_COLORING,
        display_malware_propagation(matches),
    )
}

fn input_model_path(matches: &ArgMatches) -> Option<PathBuf> {
    matches
        .get_one::<PathBuf>(ARG_INPUT_MODEL)
        .cloned()
}

fn trx_system_type(matches: &ArgMatches) -> Option<TRXSystemType> {
    let trx_system_type = match matches
        .get_one::<String>(ARG_TRX_SYSTEM) 
        .unwrap()
        .as_str() 
    {
        TRX_COLOR    => TRXSystemType::Color,
        TRX_STRENGTH => TRXSystemType::Strength,
        _ => return None,
    };

    Some(trx_system_type)
}

fn topology(matches: &ArgMatches) -> Option<Topology> {
    let topology = match matches
        .get_one::<String>(ARG_NETWORK_TOPOLOGY)
        .unwrap()
        .as_str()
    {
        TOPOLOGY_STAR => Topology::Star,
        TOPOLOGY_MESH => Topology::Mesh,
        _ => return None,
    };

    Some(topology)
}

fn drone_count(matches: &ArgMatches) -> usize {
    *matches
        .get_one::<usize>(ARG_DRONE_COUNT)
        .unwrap()
}

fn delay_multiplier(matches: &ArgMatches) -> f32 {
    *matches
        .get_one::<f32>(ARG_DELAY_MULTIPLIER)
        .unwrap()
}

fn malware(matches: &ArgMatches) -> Option<Malware> {
    let malware_type_name = matches.get_one::<String>(ARG_MALWARE_TYPE)?;
        
    let malware_type = match malware_type_name.as_str() {
        MAL_DOS       => MalwareType::DoS(DEVICE_MAX_POWER),
        MAL_INDICATOR => MalwareType::Indicator,
        _             => return None,
    };

    let malware = Malware::new(
        malware_type, 
        MALWARE_INFECTION_DELAY,
        MALWARE_SPREAD_DELAY
    );

    Some(malware)
}

fn output_directory(matches: &ArgMatches) -> Option<&Path> {
    matches
        .get_one::<PathBuf>(ARG_OUTPUT_DIR)
        .map(|p| &**p)
}

fn simulation_time(matches: &ArgMatches) -> Millisecond {
    *matches
        .get_one::<Millisecond>(ARG_SIM_TIME)
        .unwrap()
}

fn plot_caption(matches: &ArgMatches) -> &str {
    matches
        .get_one::<String>(ARG_PLOT_CAPTION)
        .unwrap()
}

fn plot_resolution(matches: &ArgMatches) -> PlotResolution {
    let plot_width = *matches
        .get_one::<Pixel>(ARG_PLOT_WIDTH)
        .unwrap();
    let plot_height = *matches
        .get_one::<Pixel>(ARG_PLOT_HEIGHT)
        .unwrap();

    PlotResolution::new(plot_width, plot_height)
}

fn display_malware_propagation(matches: &ArgMatches) -> bool {
    *matches
        .get_one::<bool>(ARG_DISPLAY_MALWARE_PROPAGATION)
        .unwrap()
}
