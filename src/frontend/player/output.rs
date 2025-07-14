use std::path::Path;

use crate::backend::mathphysics::Millisecond;
use crate::backend::networkmodel::NetworkModel;


const ERR_SERIALIZATION: &str = "Failed to serialize";


pub fn write_iteration_data(
    output_directory: Option<&Path>,
    network_model: &NetworkModel,
    current_iteration_time: Millisecond
) {
    let Some(output_directory) = output_directory else {
        return;
    };

    let local_time = chrono::Local::now()
        .format("%YY-%mm-%dd_%HH-%MM-%SS-%3ff");

    let file_name = format!("{local_time}_{current_iteration_time}");
    let file_path = output_directory.join(file_name);

    let json_data = if let Ok(data) = network_model.to_json() {
        data
    } else {
        ERR_SERIALIZATION.to_string()
    };

    let _ = std::fs::write(file_path, json_data);
}
