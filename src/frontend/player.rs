use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

use log::{info, trace};

use crate::backend::ITERATION_TIME;
use crate::backend::networkmodel::NetworkModel;
use crate::backend::mathphysics::Millisecond;

use super::renderer::PlottersRenderer;


const ERR_SERIALIZATION: &str = "Failed to serialize";


fn log_device_count(network_models: &[NetworkModel]) -> String {
    let mut output = String::new();

    for (i, network_model) in network_models.iter().enumerate() {
        let entry = format!(
            "Device count {}: {}, ", 
            i, 
            network_model.device_count()
        );
        
        output.push_str(&entry); 
    }

    // Remove trailing comma and space.
    output.truncate(output.len() - 2);

    output
}

fn write_iteration_data(
    output_directory: Option<&Path>,
    network_models: &[NetworkModel],
    current_iteration_time: Millisecond
) {
    let Some(output_directory) = output_directory else {
        return;
    };

    let local_time = chrono::Local::now()
        .format("%YY-%mm-%dd_%HH-%MM-%SS-%3ff");

    for (i, network_model) in network_models.iter().enumerate() {
        let file_name = format!("{local_time}_{current_iteration_time}i_{i}id");
        let file_path = output_directory.join(file_name);

        let json_data = if let Ok(data) = network_model.json() {
            data
        } else {
            ERR_SERIALIZATION.to_string()
        };

        let _ = fs::write(file_path, json_data);
    }
}


pub struct ModelPlayer<'a> {
    output_directory: Option<PathBuf>,
    network_models: Vec<NetworkModel>,
    renderer: PlottersRenderer<'a>,
    current_time: Millisecond,
    end_time: Millisecond,
}

impl<'a> ModelPlayer<'a> {
    #[must_use]
    pub fn new(
        output_directory: Option<&Path>,
        network_models: Vec<NetworkModel>,
        renderer: PlottersRenderer<'a>,
        end_time: Millisecond,
    ) -> Self {
        Self {
            output_directory: output_directory.map(Path::to_path_buf),
            network_models,
            renderer,
            current_time: 0,
            end_time,
        }
    }

    /// # Panics
    ///
    /// Will panic if an error occurs during rendering. 
    pub fn play(&mut self) {
        info!("Output filename: {}", self.renderer.output_filename());

        if let Some(output_directory) = &self.output_directory {
            let _ = create_dir_all(output_directory);
        }

        for _ in (0..self.end_time).step_by(ITERATION_TIME as usize) {
            write_iteration_data(
                self.output_directory.as_deref(),
                &self.network_models,
                self.current_time
            );

            self.network_models
                .iter_mut()
                .for_each(NetworkModel::update);
                
            self.renderer
                .render(&self.network_models)
                .unwrap();
                        
            trace!(
                "Current time: {}, {}", 
                self.current_time,
                log_device_count(&self.network_models)
            );

            self.current_time += ITERATION_TIME;
        }

        info!("Simulation finished at {} millis", self.current_time);
    }
}
