use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command, value_parser};

use crate::backend::connections::Topology;
use crate::backend::device::systems::TRXSystemType;
use crate::backend::malware::Malware;
use crate::backend::mathphysics::Millisecond;

use super::renderer::{Pixel, PlotResolution};

use args::{
    handle_arguments, ARG_DELAY_MULTIPLIER, ARG_DISPLAY_MALWARE_PROPAGATION, 
    ARG_DRONE_COUNT, ARG_EXPERIMENT_TITLE, ARG_MALWARE_TYPE, 
    ARG_NETWORK_TOPOLOGY, ARG_OUTPUT_DIR, ARG_PLOT_CAPTION, ARG_PLOT_HEIGHT, 
    ARG_PLOT_WIDTH, ARG_SIM_TIME, ARG_TRX_SYSTEM, DEFAULT_DELAY_MULTIPLIER, 
    DEFAULT_DRONE_COUNT, DEFAULT_PLOT_CAPTION, DEFAULT_PLOT_HEIGHT, 
    DEFAULT_PLOT_WIDTH, DEFAULT_SIM_TIME, EXP_GPS_ONLY, EXP_GPS_SPOOFING, 
    EXP_MALWARE_INFECTION, EXP_MOVEMENT, EXP_SIGNAL_LOSS, MAL_DOS, 
    MAL_INDICATOR, TOPOLOGY_BOTH, TOPOLOGY_MESH, TOPOLOGY_STAR, TRX_BOTH, 
    TRX_COLOR, TRX_STRENGTH
};


mod args;


pub fn cli() {
    let matches = Command::new("drone_network")
        .version("0.1.1")
        .about("Models drone networks.")
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
                .help("Set the plot width")
        )
        .arg(
            Arg::new(ARG_PLOT_HEIGHT)
                .long("height")
                .requires(ARG_PLOT_WIDTH)
                .value_parser(value_parser!(Pixel))
                .default_value(DEFAULT_PLOT_HEIGHT)
                .help("Set the plot height")
        )
        .arg(
            Arg::new(ARG_SIM_TIME)
                .long("time")
                .value_parser(value_parser!(Millisecond))
                .default_value(DEFAULT_SIM_TIME)
                .help("Set the simulation time")
        )
        .arg(
            Arg::new(ARG_EXPERIMENT_TITLE)
                .short('x')
                .long("experiment")
                .requires_if(EXP_MALWARE_INFECTION, ARG_MALWARE_TYPE)
                .value_parser([
                    EXP_GPS_ONLY,
                    EXP_GPS_SPOOFING,
                    EXP_MALWARE_INFECTION,
                    EXP_MOVEMENT,
                    EXP_SIGNAL_LOSS,
                ])
                .help("Choose experiment title")
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
                .help("Set the number of drones in the network")
        )
        .arg(
            Arg::new(ARG_DELAY_MULTIPLIER)
                .short('d')
                .long("delay-multiplier")
                .value_parser(value_parser!(f32))
                .default_value(DEFAULT_DELAY_MULTIPLIER)
                .help("Set signal transmission delay multiplier")
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
        .arg_required_else_help(true)
        .get_matches();

    handle_arguments(&matches);
}


pub struct GeneralConfig {
    model: ModelConfig,
    model_player: ModelPlayerConfig,
    render: RenderConfig,
}

impl GeneralConfig {
    #[must_use]
    pub fn new(
        model: ModelConfig,
        model_player: ModelPlayerConfig,
        render: RenderConfig,
    ) -> Self {
        Self {
            model,
            model_player,
            render,
        }
    }

    #[must_use]
    pub fn trx_system_type(&self) -> Option<TRXSystemType> {
        self.model.trx_system_type
    }
    
    #[must_use]
    pub fn topology(&self) -> Option<Topology> {
        self.model.topology
    }
    
    #[must_use]
    pub fn drone_count(&self) -> usize {
        self.model.drone_count
    }

    #[must_use]
    pub fn delay_multiplier(&self) -> f32 {
        self.model.delay_multiplier
    }
    
    #[must_use]
    pub fn malware(&self) -> Option<Malware> {
        self.model.malware
    }
    
    #[must_use]
    pub fn output_directory(&self) -> Option<&Path> {
        self.model_player.output_directory.as_deref()
    }
   
    #[must_use]
    pub fn simulation_time(&self) -> Millisecond {
        self.model_player.simulation_time
    }

    #[must_use]
    pub fn plot_caption(&self) -> &str {
        &self.render.plot_caption
    }
    
    #[must_use]
    pub fn plot_resolution(&self) -> PlotResolution {
        self.render.plot_resolution
    }
    
    #[must_use]
    pub fn display_malware_propagation(&self) -> bool {
        self.render.display_malware_propagation
    }
}


pub struct ModelConfig {
    // If `None`, all TRX system types will be used.
    trx_system_type: Option<TRXSystemType>,
    // If `None`, all topologies will be used.
    topology: Option<Topology>,
    drone_count: usize,
    delay_multiplier: f32,
    malware: Option<Malware>,
}

impl ModelConfig {
    #[must_use]
    pub fn new(
        trx_system_type: Option<TRXSystemType>,
        topology: Option<Topology>,
        drone_count: usize,
        delay_multiplier: f32,
        malware: Option<Malware>,
    ) -> Self {
        Self {
            trx_system_type,
            topology,
            drone_count,
            delay_multiplier,
            malware,
        }
    }
}


pub struct ModelPlayerConfig {
    output_directory: Option<PathBuf>,
    simulation_time: Millisecond,
}

impl ModelPlayerConfig {
    #[must_use]
    pub fn new(
        output_directory: Option<&Path>,
        simulation_time: Millisecond,
    ) -> Self {
        Self {
            output_directory: output_directory.map(Path::to_path_buf),
            simulation_time,
        }
    }
}


pub struct RenderConfig {
    plot_caption: String,
    plot_resolution: PlotResolution,
    display_malware_propagation: bool,
}

impl RenderConfig {
    #[must_use]
    pub fn new(
        plot_caption: &str,
        plot_resolution: PlotResolution,
        display_malware_propagation: bool,
    ) -> Self {
        Self {
            plot_caption: plot_caption.to_string(),
            plot_resolution,
            display_malware_propagation,
        }
    }
}
