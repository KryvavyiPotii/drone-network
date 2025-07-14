use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, value_parser};

use crate::backend::mathphysics::Millisecond;
use crate::frontend::renderer::Pixel;

use args::{
    handle_arguments, ARG_DELAY_MULTIPLIER, ARG_DISPLAY_MALWARE_PROPAGATION, 
    ARG_DRONE_COUNT, ARG_EXPERIMENT_TITLE, ARG_INPUT_MODEL, ARG_MALWARE_TYPE, 
    ARG_NETWORK_TOPOLOGY, ARG_OUTPUT_DIR, ARG_PLOT_CAPTION, ARG_PLOT_HEIGHT, 
    ARG_PLOT_WIDTH, ARG_SIM_TIME, ARG_TRX_SYSTEM, DEFAULT_DELAY_MULTIPLIER, 
    DEFAULT_DRONE_COUNT, DEFAULT_PLOT_CAPTION, DEFAULT_PLOT_HEIGHT, 
    DEFAULT_PLOT_WIDTH, DEFAULT_SIM_TIME, EXP_CUSTOM, EXP_GPS_ONLY, 
    EXP_GPS_SPOOFING, EXP_MALWARE_INFECTION, EXP_MOVEMENT, EXP_SIGNAL_LOSS, 
    MAL_DOS, MAL_INDICATOR, TOPOLOGY_BOTH, TOPOLOGY_MESH, TOPOLOGY_STAR, 
    TRX_BOTH, TRX_COLOR, TRX_STRENGTH
};


mod args;


pub fn cli() {
    let matches = Command::new("drone_network")
        .version("0.1.1")
        .about("Models drone networks.")
        .arg(
            Arg::new(ARG_EXPERIMENT_TITLE)
                .short('x')
                .long("experiment")
                .requires_ifs([
                    (EXP_CUSTOM, ARG_INPUT_MODEL),
                    (EXP_MALWARE_INFECTION, ARG_MALWARE_TYPE),
                ])
                .value_parser([
                    EXP_CUSTOM,
                    EXP_GPS_ONLY,
                    EXP_GPS_SPOOFING,
                    EXP_MALWARE_INFECTION,
                    EXP_MOVEMENT,
                    EXP_SIGNAL_LOSS,
                ])
                .help("Choose experiment title")
        )
        .arg(
            Arg::new(ARG_INPUT_MODEL)
                .long("im")
                .value_parser(value_parser!(PathBuf))
                .conflicts_with_all([
                    ARG_DELAY_MULTIPLIER,
                    ARG_DISPLAY_MALWARE_PROPAGATION,
                    ARG_DRONE_COUNT,
                    ARG_MALWARE_TYPE,
                    ARG_NETWORK_TOPOLOGY,
                    ARG_TRX_SYSTEM,
                ])
                .help(
                    "Deserialize network model from `.json` file and use it"
                )
        )
        .arg(
            Arg::new(ARG_TRX_SYSTEM)
                .long("trx")
                .value_parser([TRX_BOTH, TRX_COLOR, TRX_STRENGTH])
                .default_value(TRX_BOTH)
                .help("Choose device TRX system type")
        )
        .arg(
            Arg::new(ARG_NETWORK_TOPOLOGY)
                .short('t')
                .long("topology")
                .value_parser([TOPOLOGY_BOTH, TOPOLOGY_MESH, TOPOLOGY_STAR])
                .default_value(TOPOLOGY_BOTH)
                .help("Choose network topology")
        )
        .arg(
            Arg::new(ARG_DRONE_COUNT)
                .short('n')
                .value_parser(value_parser!(usize))
                .default_value(DEFAULT_DRONE_COUNT)
                .help(
                    "Set the number of drones in the network \
                    (non-negative integer)"
                )
        )
        .arg(
            Arg::new(ARG_DELAY_MULTIPLIER)
                .short('d')
                .long("delay-multiplier")
                .value_parser(value_parser!(f32))
                .default_value(DEFAULT_DELAY_MULTIPLIER)
                .help(
                    "Set signal transmission delay multiplier \
                    (non-negative float)"
                )
        )
        .arg(
            Arg::new(ARG_DISPLAY_MALWARE_PROPAGATION)
                .long("display-propagation")
                .action(ArgAction::SetTrue)
                .help(
                    format!(
                        "Show malware propagation as well \
                        (\"{EXP_MALWARE_INFECTION}\" experiment)" 
                    )
                )
        )
        .arg(
            Arg::new(ARG_MALWARE_TYPE)
                .short('i')
                .long("infection")
                .value_parser([MAL_DOS, MAL_INDICATOR])
                .help(
                    format!(
                        "Choose infection type \
                        (\"{EXP_MALWARE_INFECTION}\" experiment)"
                    )
                )
        )
        .arg(
            Arg::new(ARG_OUTPUT_DIR)
                .long("od")
                .value_parser(value_parser!(PathBuf))
                .help(
                    "Serialize network model data on each iteration to \
                    specified directory"
                )
        )
        .arg(
            Arg::new(ARG_PLOT_CAPTION)
                .short('c')
                .long("caption")
                .default_value(DEFAULT_PLOT_CAPTION)
                .help("Set the plot caption")
        )
        .arg(
            Arg::new(ARG_PLOT_WIDTH)
                .long("width")
                .requires(ARG_PLOT_HEIGHT)
                .value_parser(value_parser!(Pixel))
                .default_value(DEFAULT_PLOT_WIDTH)
                .help("Set the plot width (in pixels)")
        )
        .arg(
            Arg::new(ARG_PLOT_HEIGHT)
                .long("height")
                .requires(ARG_PLOT_WIDTH)
                .value_parser(value_parser!(Pixel))
                .default_value(DEFAULT_PLOT_HEIGHT)
                .help("Set the plot height (in pixels)")
        )
        .arg(
            Arg::new(ARG_SIM_TIME)
                .long("time")
                .value_parser(value_parser!(Millisecond))
                .default_value(DEFAULT_SIM_TIME)
                .help("Set the simulation time (in millis)")
        )
        .arg_required_else_help(true)
        .get_matches();

    handle_arguments(&matches);
}
