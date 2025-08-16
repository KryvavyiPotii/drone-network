use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, value_parser};

use crate::backend::mathphysics::Millisecond;
use crate::frontend::renderer::Pixel;

use args::{
    handle_arguments, ARG_DELAY_MULTIPLIER, ARG_DRONE_COUNT, 
    ARG_EXPERIMENT_TITLE, ARG_EW_FREQUENCY, ARG_ATTACKER_RADIUS, 
    ARG_JSON_INPUT, ARG_MALWARE_TYPE, ARG_NO_PLOT, ARG_NETWORK_TOPOLOGY, 
    ARG_JSON_OUTPUT, ARG_PLOT_CAPTION, ARG_PLOT_HEIGHT, ARG_PLOT_WIDTH, 
    ARG_SIG_LOSS_RESP, ARG_SIM_TIME, ARG_TX_MODULE, ARG_VERBOSE, 
    DEFAULT_DELAY_MULTIPLIER, DEFAULT_DRONE_COUNT, DEFAULT_PLOT_CAPTION, 
    DEFAULT_PLOT_HEIGHT, DEFAULT_PLOT_WIDTH, DEFAULT_SIM_TIME, EXP_CUSTOM, 
    EXP_EWD, EXP_GPS_SPOOFING, EXP_MALWARE_INFECTION, EXP_MOVEMENT, 
    EXP_SIGNAL_LOSS, EW_CONTROL, EW_GPS, MAL_DOS, MAL_INDICATOR, SLR_ASCEND, 
    SLR_IGNORE, SLR_HOVER, SLR_RTH, SLR_SHUTDOWN, TOPOLOGY_MESH, TOPOLOGY_STAR, 
    TX_LEVEL, TX_STRENGTH
};


mod args;


pub fn cli() {
    let matches = Command::new("drone_network")
        .version("0.2.2")
        .about("Models drone networks.")
        .args([
            arg_experiment_title(),
            arg_tx_module_type(),
            arg_signal_loss_response(),
            arg_topology(),
            arg_drone_count(),
            arg_simulation_time(),
            arg_delay_multiplier(),
            arg_ew_frequency(),
            arg_attacker_radius(),
            arg_malware_type(),
            arg_json_input(),
            arg_json_output(),
            arg_no_plot(),
            arg_plot_caption(),
            arg_plot_width(),
            arg_plot_height(),
            arg_verbose(),
        ])
        .arg_required_else_help(true)
        .get_matches();

    handle_arguments(&matches);
}

fn arg_experiment_title() -> Arg {
    Arg::new(ARG_EXPERIMENT_TITLE)
        .short('x')
        .long("experiment")
        .requires_ifs([
            (EXP_CUSTOM, ARG_JSON_INPUT),
            (EXP_MALWARE_INFECTION, ARG_MALWARE_TYPE),
        ])
        .value_parser([
            EXP_CUSTOM,
            EXP_EWD,
            EXP_GPS_SPOOFING,
            EXP_MALWARE_INFECTION,
            EXP_MOVEMENT,
            EXP_SIGNAL_LOSS,
        ])
        .help("Choose experiment title")
}

fn arg_tx_module_type() -> Arg {
    Arg::new(ARG_TX_MODULE)
        .long("tx")
        .value_parser([TX_LEVEL, TX_STRENGTH])
        .required_if_eq_any([
            (ARG_EXPERIMENT_TITLE, EXP_EWD),
            (ARG_EXPERIMENT_TITLE, EXP_GPS_SPOOFING),
            (ARG_EXPERIMENT_TITLE, EXP_MALWARE_INFECTION),
            (ARG_EXPERIMENT_TITLE, EXP_MOVEMENT),
            (ARG_EXPERIMENT_TITLE, EXP_SIGNAL_LOSS),
        ])
        .help("Choose TX system type")
}

fn arg_signal_loss_response() -> Arg {
    Arg::new(ARG_SIG_LOSS_RESP)
        .long("slr")
        .value_parser(
            [SLR_ASCEND, SLR_IGNORE, SLR_HOVER, SLR_RTH, SLR_SHUTDOWN]
        )
        .default_value(SLR_IGNORE)
        .required(true)
        .conflicts_with(EXP_SIGNAL_LOSS)
        .help(
            format!(
                "Choose control signal loss response \
                (except \"{EXP_SIGNAL_LOSS}\" experiment)"
            )
        )
}

fn arg_topology() -> Arg {
    Arg::new(ARG_NETWORK_TOPOLOGY)
        .long("topology")
        .value_parser([TOPOLOGY_MESH, TOPOLOGY_STAR])
        .required_if_eq_any([
            (ARG_EXPERIMENT_TITLE, EXP_EWD),
            (ARG_EXPERIMENT_TITLE, EXP_GPS_SPOOFING),
            (ARG_EXPERIMENT_TITLE, EXP_MALWARE_INFECTION),
            (ARG_EXPERIMENT_TITLE, EXP_MOVEMENT),
            (ARG_EXPERIMENT_TITLE, EXP_SIGNAL_LOSS),
        ])
        .help("Choose network topology")
}

fn arg_drone_count() -> Arg {
    Arg::new(ARG_DRONE_COUNT)
        .short('n')
        .value_parser(value_parser!(usize))
        .default_value(DEFAULT_DRONE_COUNT)
        .help("Set the number of drones in the network (non-negative integer)")
}

fn arg_simulation_time() -> Arg {
    Arg::new(ARG_SIM_TIME)
        .long("time")
        .value_parser(value_parser!(Millisecond))
        .default_value(DEFAULT_SIM_TIME)
        .help("Set the simulation time (non-negative integer, in millis)")
}

fn arg_ew_frequency() -> Arg {
    Arg::new(ARG_EW_FREQUENCY)
        .long("ew-freq")
        .value_parser([EW_CONTROL, EW_GPS])
        .required_if_eq(ARG_EXPERIMENT_TITLE, EXP_EWD)
        .help(format!("Choose EW frequency (\"{EXP_EWD}\" experiment)"))
}

fn arg_attacker_radius() -> Arg {
    Arg::new(ARG_ATTACKER_RADIUS)
        .long("attacker-radius")
        .value_parser(value_parser!(f32))
        .required_if_eq_any([
            (ARG_EXPERIMENT_TITLE, EXP_EWD),
            (ARG_EXPERIMENT_TITLE, EXP_GPS_SPOOFING),
            (ARG_EXPERIMENT_TITLE, EXP_MALWARE_INFECTION)
        ])
        .help(
            format!(
                "Set attacker device area radius (non-negative float) \
                (\"{EXP_EWD}\", \"{EXP_GPS_SPOOFING}\" and \
                \"{EXP_MALWARE_INFECTION}\" experiments)"
            )
        )
}

fn arg_delay_multiplier() -> Arg {
    Arg::new(ARG_DELAY_MULTIPLIER)
        .short('d')
        .long("delay-multiplier")
        .value_parser(value_parser!(f32))
        .default_value(DEFAULT_DELAY_MULTIPLIER)
        .help(
            "Set signal transmission delay multiplier \
            (non-negative float)"
        )
}

fn arg_malware_type() -> Arg {
    Arg::new(ARG_MALWARE_TYPE)
        .long("mt")
        .value_parser([MAL_DOS, MAL_INDICATOR])
        .help(
            format!(
                "Choose malware type (\"{EXP_MALWARE_INFECTION}\" experiment)"
            )
        )
}

fn arg_json_input() -> Arg {
    Arg::new(ARG_JSON_INPUT)
        .long("ji")
        .value_parser(value_parser!(PathBuf))
        .conflicts_with_all([
            ARG_DELAY_MULTIPLIER,
            ARG_DRONE_COUNT,
            ARG_MALWARE_TYPE,
            ARG_NETWORK_TOPOLOGY,
            ARG_TX_MODULE,
        ])
        .help(
            format!(
                "Deserialize network model from `.json` file and use it \
                (\"{EXP_CUSTOM}\" experiment)"
            )
        )
}

fn arg_json_output() -> Arg {
    Arg::new(ARG_JSON_OUTPUT)
        .long("jo")
        .value_parser(value_parser!(PathBuf))
        .help(
            "Serialize network model data on each iteration to `.json` files \
            in specified directory"
        )
}

fn arg_no_plot() -> Arg {
    Arg::new(ARG_NO_PLOT)
        .long("no-plot")
        .action(ArgAction::SetTrue)
        .help("Do not render a GIF plot")
}

fn arg_plot_caption() -> Arg {
    Arg::new(ARG_PLOT_CAPTION)
        .short('c')
        .long("caption")
        .default_value(DEFAULT_PLOT_CAPTION)
        .help("Set the plot caption")
}

fn arg_plot_width() -> Arg {
    Arg::new(ARG_PLOT_WIDTH)
        .long("width")
        .requires(ARG_PLOT_HEIGHT)
        .value_parser(value_parser!(Pixel))
        .default_value(DEFAULT_PLOT_WIDTH)
        .help("Set the plot width (in pixels)")
}

fn arg_plot_height() -> Arg {
    Arg::new(ARG_PLOT_HEIGHT)
        .long("height")
        .requires(ARG_PLOT_WIDTH)
        .value_parser(value_parser!(Pixel))
        .default_value(DEFAULT_PLOT_HEIGHT)
        .help("Set the plot height (in pixels)")
}

fn arg_verbose() -> Arg {
    Arg::new(ARG_VERBOSE)
        .short('v')
        .long("verbose")
        .action(ArgAction::SetTrue)
        .help("Show full log output")
}
