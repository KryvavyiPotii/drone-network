use std::path::{Path, PathBuf};

use log::info;

use crate::backend::ITERATION_TIME;
use crate::backend::networkmodel::NetworkModel;
use crate::backend::mathphysics::Millisecond;

use super::renderer::PlottersRenderer;

use output::write_iteration_data;


mod output;


pub struct ModelPlayer<'a> {
    output_directory: Option<PathBuf>,
    network_model: NetworkModel,
    renderer: PlottersRenderer<'a>,
    current_time: Millisecond,
    end_time: Millisecond,
}

impl<'a> ModelPlayer<'a> {
    #[must_use]
    pub fn new(
        output_directory: Option<&Path>,
        network_model: NetworkModel,
        renderer: PlottersRenderer<'a>,
        end_time: Millisecond,
    ) -> Self {
        Self {
            output_directory: output_directory.map(Path::to_path_buf),
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

        if let Some(output_directory) = &self.output_directory {
            let _ = std::fs::create_dir_all(output_directory);
        }

        for _ in (0..self.end_time).step_by(ITERATION_TIME as usize) {
            write_iteration_data(
                self.output_directory.as_deref(),
                &self.network_model,
                self.current_time
            );

            self.network_model.update();

            self.renderer.render(&self.network_model);
                        
            self.current_time += ITERATION_TIME;
        }

        self.end_info();
    }

    fn start_info(&self) {
        info!(
            "Rendering in {}", 
            self.renderer.output_filename()
        );
        info!(
            "Initial device count: {}", 
            self.network_model.device_map().len()
        );
    }

    fn end_info(&self) {
        info!(
            "Simulation finished at {}", 
            self.current_time
        );
        info!(
            "Conclusive device count: {}", 
            self.network_model.device_map().len()
        );
        info!(
            "Render filename: {}", 
            self.renderer.output_filename()
        );
    }
}
