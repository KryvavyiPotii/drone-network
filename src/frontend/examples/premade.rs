use crate::backend::connections::Topology;
use crate::backend::device::{
    Device, DeviceBuilder, SignalLossResponse, device_map_from_slice,
};
use crate::backend::device::systems::TXModuleType;
use crate::backend::malware::{Malware, MalwareType};
use crate::backend::mathphysics::{Frequency, Meter, Point3D};
use crate::backend::networkmodel::NetworkModelBuilder; 
use crate::backend::networkmodel::attack::{AttackType, AttackerDevice};
use crate::backend::signal::{
    SignalQuality, GREEN_SIGNAL_QUALITY, RED_SIGNAL_QUALITY
};
use crate::frontend::config::GeneralConfig;
use crate::frontend::player::ModelPlayer;
use crate::frontend::renderer::{
    Axes3DRanges, CameraAngle, DeviceColoring, PlottersRenderer, 
    DEFAULT_AXES_RANGE, DEFAULT_CAMERA_ANGLE, DEFAULT_DEVICE_COLORING
};

use devsetup::{
    attack_scenario, cc_trx_system, create_drone_vec, default_gps, 
    default_network_position, device_movement_system, device_power_system, 
    drone_trx_system, ewd_trx_system, indicator_malware, reposition_scenario,
    CC_POSITION, NETWORK_ORIGIN
};


pub use devsetup::DEVICE_MAX_POWER;


mod devsetup;


fn derive_filename(
    tx_module_type: TXModuleType, 
    topology: Topology,
    text: &str
) -> String {
    let tx_module_part = match tx_module_type {
        TXModuleType::Level    => "lvl",
        TXModuleType::Strength => "str",
    };
    let topology_part = match topology {
        Topology::Mesh => "mesh",
        Topology::Star => "star",
    };

    format!("{tx_module_part}_{text}_{topology_part}.gif")
}


pub fn ewd(
    general_config: &GeneralConfig, 
    ew_frequency: Frequency,
    ewd_area_radius: Meter,
) {
    let cc_tx_control_area_radius    = 200.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_quality  = RED_SIGNAL_QUALITY; 
        
    let command_center = DeviceBuilder::new()
        .set_real_position(CC_POSITION)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().tx_module_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        &default_network_position(NETWORK_ORIGIN),
        None,
        general_config.model_config().tx_module_type(),
        general_config.model_config().signal_loss_response(),
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_quality, 
    );
    devices.insert(0, command_center);
 
    let ewd = DeviceBuilder::new()
        .set_real_position(Point3D::new(0.0, 5.0, 2.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().tx_module_type(), 
                ew_frequency, 
                ewd_area_radius
            )
        )
        .build();
    let attacker_devices = vec![
        AttackerDevice::new(ewd, AttackType::ElectronicWarfare)
    ];

    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(device_map_from_slice(devices.as_slice()))
        .set_attacker_devices(attacker_devices)
        .set_gps(default_gps(general_config.model_config().tx_module_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();

    let renderer = general_config
        .model_player_config()
        .render_config()
        .map(|render_config| { 
            let output_filename = derive_filename(
                general_config.model_config().tx_module_type(), 
                general_config.model_config().topology(), 
                "ewd"
            );
            
            PlottersRenderer::new(
                &output_filename,
                render_config.plot_caption(),
                render_config.plot_resolution(),
                DEFAULT_AXES_RANGE,
                DEFAULT_DEVICE_COLORING,
                DEFAULT_CAMERA_ANGLE,
            )
        });

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().json_output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn movement(general_config: &GeneralConfig) {
    let cc_tx_control_area_radius    = 300.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_quality  = SignalQuality::from(10_000.0); 

    let command_center = DeviceBuilder::new()
        .set_real_position(CC_POSITION)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().tx_module_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        &default_network_position(NETWORK_ORIGIN),
        None,
        general_config.model_config().tx_module_type(),
        general_config.model_config().signal_loss_response(),
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_quality, 
    );
    devices.insert(0, command_center);
    
    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(device_map_from_slice(devices.as_slice()))
        .set_gps(default_gps(general_config.model_config().tx_module_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(reposition_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();

    let renderer = general_config
        .model_player_config()
        .render_config()
        .map(|render_config| { 
            let output_filename = derive_filename(
                general_config.model_config().tx_module_type(),
                general_config.model_config().topology(), 
                "movement"
            );
                    
            PlottersRenderer::new(
                &output_filename,
                render_config.plot_caption(),
                render_config.plot_resolution(),
                DEFAULT_AXES_RANGE,
                DEFAULT_DEVICE_COLORING,
                DEFAULT_CAMERA_ANGLE,
            )
        });

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().json_output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn gps_spoofing(
    general_config: &GeneralConfig,
    spoofer_area_radius: Meter
) {
    let cc_tx_control_area_radius    = 300.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_quality  = RED_SIGNAL_QUALITY; 
        
    let command_center = DeviceBuilder::new()
        .set_real_position(CC_POSITION)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().tx_module_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        &default_network_position(NETWORK_ORIGIN),
        None,
        general_config.model_config().tx_module_type(),
        general_config.model_config().signal_loss_response(),
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_quality, 
    );
    devices.insert(0, command_center);

    let spoofer = DeviceBuilder::new()
        .set_real_position(Point3D::new(0.0, 5.0, 2.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().tx_module_type(), 
                Frequency::GPS, 
                spoofer_area_radius
            )
        )
        .build();
    let spoofed_position = Point3D::new(-200.0, -100.0, -200.0);
    let attacker_devices = vec![
        AttackerDevice::new(spoofer, AttackType::GPSSpoofing(spoofed_position))
    ];

    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(device_map_from_slice(devices.as_slice()))
        .set_attacker_devices(attacker_devices)
        .set_gps(default_gps(general_config.model_config().tx_module_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();

    let renderer = general_config
        .model_player_config()
        .render_config()
        .map(|render_config| { 
            let output_filename = derive_filename(
                general_config.model_config().tx_module_type(),
                general_config.model_config().topology(), 
                "gps_spoofing"
            );
            let axes_ranges = Axes3DRanges::new(
                0.0..200.0, 
                0.0..0.0, 
                0.0..200.0
            );
            let camera_angle = CameraAngle::new(1.57, 1.57);

            PlottersRenderer::new(
                &output_filename,
                render_config.plot_caption(),
                render_config.plot_resolution(),
                axes_ranges,
                DEFAULT_DEVICE_COLORING,
                camera_angle,
            )
        });

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().json_output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn malware_infection(
    general_config: &GeneralConfig,
    malware: Malware,
    attacker_area_radius: Meter,
    display_malware_propagation: bool
) {
    let cc_tx_control_area_radius    = 200.0;
    let drone_tx_control_area_radius = 15.0;
    let drone_gps_rx_signal_quality  = GREEN_SIGNAL_QUALITY; 

    let command_center = DeviceBuilder::new()
        .set_real_position(Point3D::new(100.0, 50.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().tx_module_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        &default_network_position(Point3D::new(50.0, 50.0, 0.0)),
        Some(malware),
        general_config.model_config().tx_module_type(),
        general_config.model_config().signal_loss_response(),
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_quality, 
    );
    devices.insert(0, command_center);
    
    let attacker = DeviceBuilder::new()
        .set_real_position(Point3D::new(-10.0, 2.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().tx_module_type(),
                Frequency::Control,
                attacker_area_radius
            )
        )
        .build();
    let attacker_devices = vec![
        AttackerDevice::new(
            attacker.clone(), 
            AttackType::MalwareDistribution(malware)
        )
    ];

    let drone_network_builder = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(device_map_from_slice(devices.as_slice()))
        .set_gps(default_gps(general_config.model_config().tx_module_type()))
        .set_topology(general_config.model_config().topology())
        .set_delay_multiplier(general_config.model_config().delay_multiplier());
    
    if display_malware_propagation {
        malware_propagation(
            attacker,
            drone_network_builder.clone(),
            general_config,
        );
    }

    let drone_network = drone_network_builder
        .set_attacker_devices(attacker_devices)
        .build();

    let renderer = general_config
        .model_player_config()
        .render_config()
        .map(|render_config| { 
            let text = match malware.malware_type() {
                MalwareType::DoS(_)     => "mal_dos",
                MalwareType::Indicator  => "mal_indicator",
            };
            let output_filename = derive_filename(
                general_config.model_config().tx_module_type(),
                general_config.model_config().topology(), 
                text,
            );
            let drone_coloring = match malware.malware_type() {
                MalwareType::DoS(_)    => DeviceColoring::SingleColor(0, 0, 0),
                MalwareType::Indicator => DeviceColoring::Infection,
            };
            let axes_ranges = Axes3DRanges::new(
                0.0..100.0, 
                0.0..0.0, 
                0.0..100.0
            );
            let camera_angle = CameraAngle::new(1.57, 1.57);
                    
            PlottersRenderer::new(
                &output_filename,
                render_config.plot_caption(),
                render_config.plot_resolution(),
                axes_ranges,
                drone_coloring,
                camera_angle
            )
        });

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().json_output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn malware_propagation(
    attacker: Device,
    drone_network_builder: NetworkModelBuilder,
    general_config: &GeneralConfig,
) {
    let malware = indicator_malware();
    let attacker_devices = vec![
        AttackerDevice::new(
            attacker, 
            AttackType::MalwareDistribution(malware)
        )
    ];

    let drone_network = drone_network_builder
        .set_attacker_devices(attacker_devices)
        .build();

    let renderer = general_config
        .model_player_config()
        .render_config()
        .map(|render_config| { 
            let output_filename = derive_filename(
                general_config.model_config().tx_module_type(),
                general_config.model_config().topology(), 
                "mal_indicator",
            );
            let drone_coloring = DeviceColoring::Infection; 
            let axes_ranges = Axes3DRanges::new(
                0.0..100.0, 
                0.0..0.0,
                0.0..100.0
            );
            let camera_angle = CameraAngle::new(1.57, 1.57);

            PlottersRenderer::new(
                &output_filename,
                render_config.plot_caption(),
                render_config.plot_resolution(),
                axes_ranges,
                drone_coloring,
                camera_angle,
            )
        });

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().json_output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn signal_loss_response(general_config: &GeneralConfig) {
    let cc_tx_control_area_radius    = 200.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_quality  = GREEN_SIGNAL_QUALITY; 
    let control_ewd_suppression_area_radius = 25.0;
    let command_center_position      = Point3D::new(100.0, 50.0, 0.0);

    let command_center = DeviceBuilder::new()
        .set_real_position(command_center_position)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().tx_module_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();
   
    let drone_builder = DeviceBuilder::new()
        .set_real_position(Point3D::new(70.0, 50.0, 30.0))
        .set_power_system(device_power_system())
        .set_movement_system(device_movement_system())
        .set_trx_system(
            drone_trx_system(
                general_config.model_config().tx_module_type(), 
                drone_tx_control_area_radius, 
                drone_gps_rx_signal_quality
            )
        );

    let ascend_drone = drone_builder
        .clone()
        .set_signal_loss_response(SignalLossResponse::Ascend)
        .build();
    let hover_drone = drone_builder
        .clone()
        .set_signal_loss_response(SignalLossResponse::Hover)
        .build();
    let ignore_drone = drone_builder
        .clone()
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let rth_drone = drone_builder
        .clone()
        .set_signal_loss_response(
            SignalLossResponse::ReturnToHome(command_center_position)
        )
        .build();
    let shutdown_drone = drone_builder
        .set_signal_loss_response(SignalLossResponse::Shutdown)
        .build();
    let devices = [
        command_center, 
        ascend_drone, 
        hover_drone, 
        ignore_drone,
        rth_drone, 
        shutdown_drone
    ]; 
    
    let ewd_control = DeviceBuilder::new()
        .set_real_position(Point3D::new(-10.0, 2.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().tx_module_type(),
                Frequency::Control,
                control_ewd_suppression_area_radius
            )
        )
        .build();
    let attacker_devices = vec![
        AttackerDevice::new(ewd_control, AttackType::ElectronicWarfare)
    ];
    
    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(device_map_from_slice(devices.as_slice()))
        .set_attacker_devices(attacker_devices)
        .set_gps(default_gps(general_config.model_config().tx_module_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();
 
    let renderer = general_config
        .model_player_config()
        .render_config()
        .map(|render_config| { 
            let output_filename = derive_filename(
                general_config.model_config().tx_module_type(),
                general_config.model_config().topology(),
                "signal_loss_response"
            ); 
            let axes_ranges = Axes3DRanges::new(
                0.0..100.0, 
                0.0..100.0, 
                0.0..100.0
            );

            PlottersRenderer::new(
                &output_filename,
                render_config.plot_caption(),
                render_config.plot_resolution(),
                axes_ranges,
                DEFAULT_DEVICE_COLORING,
                DEFAULT_CAMERA_ANGLE,
            )
        });
    
    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().json_output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}
