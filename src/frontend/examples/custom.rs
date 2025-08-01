use std::path::Path;

use crate::backend::networkmodel::NetworkModel;
use crate::frontend::config::ModelPlayerConfig;
use crate::frontend::player::ModelPlayer;
use crate::frontend::renderer::PlottersRenderer;


pub fn custom(
    network_model_path: &Path,
    model_player_config: &ModelPlayerConfig,
) {
    let network_model = NetworkModel::from_json(network_model_path)
        .expect("Failed to deserialize network model");

    let renderer = model_player_config
        .render_config() 
        .map(|render_config|
            PlottersRenderer::new(
                "custom.gif",
                render_config.plot_caption(),
                render_config.plot_resolution(),
                render_config.axes_ranges(),
                render_config.device_coloring(),
                render_config.camera_angle()
            )
        );

    let mut model_player = ModelPlayer::new(
        model_player_config.json_output_directory(),
        network_model,
        renderer,
        model_player_config.simulation_time(),
    );

    model_player.play();
}
