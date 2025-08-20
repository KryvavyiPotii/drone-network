use std::io::Write;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use env_logger::{Builder, Target};
use log::LevelFilter;

use crate::backend::connections::Topology;
use crate::backend::device::SignalLossResponse;
use crate::backend::malware::{Malware, MalwareType};
use crate::backend::mathphysics::{Frequency, Millisecond, Point3D};
use crate::frontend::{MALWARE_INFECTION_DELAY, MALWARE_SPREAD_DELAY};
use crate::frontend::config::{
    GeneralConfig, ModelConfig, ModelPlayerConfig, RenderConfig
};
use crate::frontend::examples::{Example, DEVICE_MAX_POWER};
use crate::frontend::renderer::{
    CameraAngle, Pixel, PlottersUnit, PlotResolution, DEFAULT_AXES_RANGE, 
    DEFAULT_DEVICE_COLORING
};


pub const ARG_ATTACKER_RADIUS: &str  = "attacker device area radius";
pub const ARG_CAMERA_PITCH: &str     = "camera pitch";
pub const ARG_CAMERA_YAW: &str       = "camera yaw";
pub const ARG_DELAY_MULTIPLIER: &str = "delay multiplier";
pub const ARG_DRONE_COUNT: &str      = "drone count";
pub const ARG_EXPERIMENT_TITLE: &str = "experiment title";
pub const ARG_EW_FREQUENCY: &str     = "electronic warfare frequency";
pub const ARG_JSON_INPUT: &str       = "json input path";
pub const ARG_JSON_OUTPUT: &str      = "json directory output path";
pub const ARG_MALWARE_TYPE: &str     = "malware type";
pub const ARG_NETWORK_TOPOLOGY: &str = "network topology";
pub const ARG_NO_PLOT: &str          = "no GIF rendering";
pub const ARG_PLOT_CAPTION: &str     = "plot caption";
pub const ARG_PLOT_HEIGHT: &str      = "plot height";
pub const ARG_PLOT_WIDTH: &str       = "plot width";
pub const ARG_SIG_LOSS_RESP: &str    = "control signal loss response"; 
pub const ARG_SIM_TIME: &str         = "simulation time";
pub const ARG_VERBOSE: &str          = "verbose logs";

pub const EXP_CUSTOM: &str            = "custom";
pub const EXP_EWD: &str               = "ewd";
pub const EXP_GPS_SPOOFING: &str      = "gpsspoof";
pub const EXP_MALWARE_INFECTION: &str = "malware";
pub const EXP_MOVEMENT: &str          = "move";
pub const EXP_SIGNAL_LOSS: &str       = "signalloss";

pub const EW_CONTROL: &str = "control";
pub const EW_GPS: &str     = "gps";

pub const MAL_DOS: &str       = "dos";
pub const MAL_INDICATOR: &str = "indicator";

pub const SLR_ASCEND: &str   = "ascend";
pub const SLR_IGNORE: &str   = "ignore";
pub const SLR_HOVER: &str    = "hover";
pub const SLR_RTH: &str      = "rth"; // Return to command center.
pub const SLR_SHUTDOWN: &str = "shutdown"; 

pub const TOPOLOGY_MESH: &str = "mesh";
pub const TOPOLOGY_STAR: &str = "star";

pub const DEFAULT_CAMERA_PITCH: &str     = "0.15";
pub const DEFAULT_CAMERA_YAW: &str       = "0.5";
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
        EXP_CUSTOM            =>
            Example::Custom(input_model_path(matches)),
        EXP_EWD               => 
            Example::EWD {
                ew_frequency: ew_frequency(matches), 
                ewd_area_radius: attacker_radius(matches)
            },
        EXP_GPS_SPOOFING      => 
            Example::GPSSpoofing { 
                spoofer_area_radius: attacker_radius(matches) 
            },
        EXP_MALWARE_INFECTION => 
            Example::MalwareInfection {
                malware: malware(matches),
                attacker_area_radius: attacker_radius(matches),
            }, 
        EXP_MOVEMENT          => Example::Movement,
        EXP_SIGNAL_LOSS       => Example::SignalLossResponse,
        _                     => return
    };

    let model_config = match example {
        Example::Custom(_) => ModelConfig::default(),
        _                  => model_config(matches),
    };
    
    configure_logging(verbosity_level(matches));
    
    example.execute(
        &GeneralConfig::new(
            model_config,
            model_player_config(matches),
        )
    );
}

fn model_config(matches: &ArgMatches) -> ModelConfig {
    ModelConfig::new(
        signal_loss_response(matches),
        topology(matches),
        drone_count(matches),
        delay_multiplier(matches),
    )
}

fn model_player_config(matches: &ArgMatches) -> ModelPlayerConfig {
    let render_config = if no_rendering(matches) {
        None
    } else {
        Some(render_config(matches))
    };

    ModelPlayerConfig::new(
        json_output_directory(matches), 
        render_config,
        simulation_time(matches),
    )
}

fn render_config(matches: &ArgMatches) -> RenderConfig {
    RenderConfig::new(
        plot_caption(matches), 
        plot_resolution(matches), 
        DEFAULT_AXES_RANGE,
        camera_angle(matches), 
        DEFAULT_DEVICE_COLORING,
    )
}

fn input_model_path(matches: &ArgMatches) -> PathBuf {
    matches
        .get_one::<PathBuf>(ARG_JSON_INPUT)
        .unwrap()
        .clone()
}

fn ew_frequency(matches: &ArgMatches) -> Frequency {
    match matches
        .get_one::<String>(ARG_EW_FREQUENCY) 
        .unwrap()
        .as_str() 
    {
        EW_CONTROL  => Frequency::Control,
        EW_GPS      => Frequency::GPS,
        _           => panic!("Wrong EW frequency")
    }
}

fn attacker_radius(matches: &ArgMatches) -> f32 {
    *matches
        .get_one::<f32>(ARG_ATTACKER_RADIUS)
        .unwrap()
}

fn signal_loss_response(matches: &ArgMatches) -> SignalLossResponse {
    match matches
        .get_one::<String>(ARG_SIG_LOSS_RESP) 
        .unwrap()
        .as_str() 
    {   
        SLR_ASCEND   => SignalLossResponse::Ascend,
        SLR_IGNORE   => SignalLossResponse::Ignore,
        SLR_HOVER    => SignalLossResponse::Hover,
        SLR_RTH      => SignalLossResponse::ReturnToHome(Point3D::default()),
        SLR_SHUTDOWN => SignalLossResponse::Shutdown,
        _            => panic!("Wrong signal loss response")
    }
}

fn topology(matches: &ArgMatches) -> Topology {
    match matches
        .get_one::<String>(ARG_NETWORK_TOPOLOGY)
        .unwrap()
        .as_str()
    {
        TOPOLOGY_STAR => Topology::Star,
        TOPOLOGY_MESH => Topology::Mesh,
        _             => panic!("Wrong topology")
    }
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

fn malware(matches: &ArgMatches) -> Malware {
    let malware_type = match matches
        .get_one::<String>(ARG_MALWARE_TYPE)
        .unwrap()
        .as_str() 
    {
        MAL_DOS       => MalwareType::DoS(DEVICE_MAX_POWER),
        MAL_INDICATOR => MalwareType::Indicator,
        _             => panic!("Wrong malware type"),
    };

    Malware::new(
        malware_type, 
        MALWARE_INFECTION_DELAY,
        MALWARE_SPREAD_DELAY
    )
}

fn json_output_directory(matches: &ArgMatches) -> Option<&Path> {
    matches
        .get_one::<PathBuf>(ARG_JSON_OUTPUT)
        .map(|p| &**p)
}

fn simulation_time(matches: &ArgMatches) -> Millisecond {
    *matches
        .get_one::<Millisecond>(ARG_SIM_TIME)
        .unwrap()
}

fn no_rendering(matches: &ArgMatches) -> bool {
    *matches
        .get_one::<bool>(ARG_NO_PLOT)
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

fn camera_angle(matches: &ArgMatches) -> CameraAngle {
    let camera_pitch = *matches
        .get_one::<PlottersUnit>(ARG_CAMERA_PITCH)
        .unwrap();
    let camera_yaw = *matches
        .get_one::<PlottersUnit>(ARG_CAMERA_YAW)
        .unwrap();

    CameraAngle::new(camera_pitch, camera_yaw)
}

fn verbosity_level(matches: &ArgMatches) -> LevelFilter {
    if *matches.get_one::<bool>(ARG_VERBOSE).unwrap() {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    }
}

fn configure_logging(filter: LevelFilter) {
    Builder::new()
        .format(|buf, record| 
            writeln!(
                buf,
                "{} {} - {}", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(), 
                record.args()
            )
        )
        .filter(None, filter)
        .target(Target::Stdout)
        .init();
}
