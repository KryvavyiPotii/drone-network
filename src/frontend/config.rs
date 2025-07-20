use std::path::{Path, PathBuf};

use crate::backend::connections::Topology;
use crate::backend::device::systems::TRXSystemType;
use crate::backend::malware::Malware;
use crate::backend::mathphysics::Millisecond;

use crate::frontend::renderer::{
    Axes3DRanges, CameraAngle, DeviceColoring, PlotResolution
};


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
    pub fn model_config(&self) -> &ModelConfig {
        &self.model
    }
    
    #[must_use]
    pub fn model_player_config(&self) -> &ModelPlayerConfig {
        &self.model_player
    }
    
    #[must_use]
    pub fn render_config(&self) -> &RenderConfig {
        &self.render
    }

    #[must_use]
    pub fn malware_list(&self, indicator_malware: Malware) -> Vec<Malware> {
        match self.model.malware {
            Some(malware) if self.render.display_malware_propagation =>
                vec![malware, indicator_malware],
            Some(malware) => vec![malware],
            None          => Vec::new()
        }
    }
}


#[derive(Default)]
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

    #[must_use]
    pub fn trx_system_type(&self) -> Option<TRXSystemType> {
        self.trx_system_type
    }
    
    #[must_use]
    pub fn topology(&self) -> Option<Topology> {
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
    
    #[must_use]
    pub fn malware(&self) -> Option<Malware> {
        self.malware
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
    
    #[must_use]
    pub fn output_directory(&self) -> Option<&Path> {
        self.output_directory.as_deref()
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
    display_malware_propagation: bool,
}

impl RenderConfig {
    #[must_use]
    pub fn new(
        plot_caption: &str,
        plot_resolution: PlotResolution,
        axes_ranges: Axes3DRanges,
        camera_angle: CameraAngle,
        device_coloring: DeviceColoring,
        display_malware_propagation: bool,
    ) -> Self {
        Self {
            plot_caption: plot_caption.to_string(),
            plot_resolution,
            axes_ranges,
            camera_angle,
            device_coloring,
            display_malware_propagation,
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
    
    #[must_use]
    pub fn display_malware_propagation(&self) -> bool {
        self.display_malware_propagation
    }
}
