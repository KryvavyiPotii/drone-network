use std::path::{Path, PathBuf};

use log::info;

use crate::backend::ITERATION_TIME;
use crate::backend::networkmodel::NetworkModel;
use crate::backend::mathphysics::Millisecond;

use super::renderer::PlottersRenderer;

use output::write_iteration_data;


mod output;


pub struct ModelPlayer<'a> {
    json_output_directory: Option<PathBuf>,
    network_model: NetworkModel,
    renderer: Option<PlottersRenderer<'a>>,
    current_time: Millisecond,
    end_time: Millisecond,
}

impl<'a> ModelPlayer<'a> {
    #[must_use]
    pub fn new(
        json_output_directory: Option<&Path>,
        network_model: NetworkModel,
        renderer: Option<PlottersRenderer<'a>>,
        end_time: Millisecond,
    ) -> Self {
        Self {
            json_output_directory: json_output_directory.map(Path::to_path_buf),
            network_model,
            renderer,
            current_time: 0,
            end_time,
        }
    }

    /// # Panics
    ///
    /// Will panic if an error occurs during rendering. 
    pub fn play(&mut self) {
        self.start_info();

        if let Some(json_output_directory) = &self.json_output_directory {
            let _ = std::fs::create_dir_all(json_output_directory);
        }

        for _ in (0..self.end_time).step_by(ITERATION_TIME as usize) {
            info!("Current time: {}", self.current_time);

            if let Some(
                ref json_output_directory
            ) = self.json_output_directory {
                write_iteration_data(
                    json_output_directory,
                    &self.network_model,
                    self.current_time
                );
            }

            self.network_model.update();

            if let Some(ref mut renderer) = self.renderer {
                renderer.render(&self.network_model);
            }
                        
            self.current_time += ITERATION_TIME;
        }

        self.end_info();
    }

    fn start_info(&self) {
        self.renderer
            .as_ref()
            .inspect(|renderer| {
                info!("Rendering in {}", renderer.output_filename());
            });
        info!(
            "Initial device count: {}", 
            self.network_model.device_map().len()
        );
    }

    fn end_info(&self) {
        info!("Simulation finished at {}", self.current_time);
        info!(
            "Conclusive device count: {}", 
            self.network_model.device_map().len()
        );
        self.renderer
            .as_ref()
            .inspect(|renderer| {
                info!("Render filename: {}", renderer.output_filename());
            });
    }
}
