use std::path::{Path, PathBuf};

use crate::backend::connections::Topology;
use crate::backend::device::SignalLossResponse;
use crate::backend::device::systems::TXModuleType;
use crate::backend::mathphysics::Millisecond;

use crate::frontend::renderer::{
    Axes3DRanges, CameraAngle, DeviceColoring, PlotResolution
};


pub struct GeneralConfig {
    model_config: ModelConfig,
    model_player_config: ModelPlayerConfig,
}

impl GeneralConfig {
    #[must_use]
    pub fn new(
        model_config: ModelConfig,
        model_player_config: ModelPlayerConfig,
    ) -> Self {
        Self {
            model_config,
            model_player_config,
        }
    }
    
    #[must_use]
    pub fn model_config(&self) -> &ModelConfig {
        &self.model_config
    }
    
    #[must_use]
    pub fn model_player_config(&self) -> &ModelPlayerConfig {
        &self.model_player_config
    }
}


#[derive(Default)]
pub struct ModelConfig {
    tx_module_type: TXModuleType,
    signal_loss_response: SignalLossResponse,
    topology: Topology,
    drone_count: usize,
    delay_multiplier: f32,
}

impl ModelConfig {
    #[must_use]
    pub fn new(
        tx_module_type: TXModuleType,
        signal_loss_response: SignalLossResponse,
        topology: Topology,
        drone_count: usize,
        delay_multiplier: f32,
    ) -> Self {
        Self {
            tx_module_type,
            signal_loss_response,
            topology,
            drone_count,
            delay_multiplier,
        }
    }

    #[must_use]
    pub fn tx_module_type(&self) -> TXModuleType {
        self.tx_module_type
    }
    
    #[must_use]
    pub fn signal_loss_response(&self) -> SignalLossResponse {
        self.signal_loss_response
    }
    
    #[must_use]
    pub fn topology(&self) -> Topology {
        self.topology
    }
    
    #[must_use]
    pub fn drone_count(&self) -> usize {
        self.drone_count
    }

    #[must_use]
    pub fn delay_multiplier(&self) -> f32 {
        self.delay_multiplier
    }
}


pub struct ModelPlayerConfig {
    json_output_directory: Option<PathBuf>,
    render_config: Option<RenderConfig>,
    simulation_time: Millisecond,
}

impl ModelPlayerConfig {
    #[must_use]
    pub fn new(
        json_output_directory: Option<&Path>,
        render_config: Option<RenderConfig>,
        simulation_time: Millisecond,
    ) -> Self {
        Self {
            json_output_directory: json_output_directory.map(Path::to_path_buf),
            render_config,
            simulation_time,
        }
    }
    
    #[must_use]
    pub fn json_output_directory(&self) -> Option<&Path> {
        self.json_output_directory.as_deref()
    }

    #[must_use]
    pub fn render_config(&self) -> Option<&RenderConfig> {
        self.render_config.as_ref()
    }
   
    #[must_use]
    pub fn simulation_time(&self) -> Millisecond {
        self.simulation_time
    }
}


pub struct RenderConfig {
    plot_caption: String,
    plot_resolution: PlotResolution,
    axes_ranges: Axes3DRanges,
    camera_angle: CameraAngle,
    device_coloring: DeviceColoring,
}

impl RenderConfig {
    #[must_use]
    pub fn new(
        plot_caption: &str,
        plot_resolution: PlotResolution,
        axes_ranges: Axes3DRanges,
        camera_angle: CameraAngle,
        device_coloring: DeviceColoring,
    ) -> Self {
        Self {
            plot_caption: plot_caption.to_string(),
            plot_resolution,
            axes_ranges,
            camera_angle,
            device_coloring,
        }
    }
    
    #[must_use]
    pub fn plot_caption(&self) -> &str {
        &self.plot_caption
    }
    
    #[must_use]
    pub fn plot_resolution(&self) -> PlotResolution {
        self.plot_resolution
    }
    
    #[must_use]
    pub fn axes_ranges(&self) -> Axes3DRanges {
        self.axes_ranges.clone()
    }
    
    #[must_use]
    pub fn camera_angle(&self) -> CameraAngle {
        self.camera_angle
    }
    
    #[must_use]
    pub fn device_coloring(&self) -> DeviceColoring {
        self.device_coloring
    }
}
