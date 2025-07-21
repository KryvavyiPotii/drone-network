use crate::backend::CONTROL_FREQUENCY;
use crate::backend::connections::Topology;
use crate::backend::device::{
    Device, DeviceBuilder, IdToDeviceMap, SignalLossResponse, BROADCAST_ID, 
};
use crate::backend::device::systems::TRXSystemType;
use crate::backend::malware::{Malware, MalwareType};
use crate::backend::mathphysics::Point3D;
use crate::backend::networkmodel::NetworkModelBuilder; 
use crate::backend::networkmodel::attack::{AttackType, AttackerDevice};
use crate::backend::signal::{
    SignalLevel, GPS_L1_FREQUENCY, GREEN_SIGNAL_LEVEL, RED_SIGNAL_LEVEL
};
use crate::backend::task::{Scenario, Task, TaskType};
use crate::frontend::{MALWARE_INFECTION_DELAY, MALWARE_SPREAD_DELAY};
use crate::frontend::config::GeneralConfig;
use crate::frontend::player::ModelPlayer;
use crate::frontend::renderer::{
    Axes3DRanges, CameraAngle, DeviceColoring, PlottersRenderer, 
    DEFAULT_AXES_RANGE, DEFAULT_CAMERA_ANGLE, DEFAULT_DEVICE_COLORING
};

use super::devsetup::{
    cc_trx_system, create_drone_vec, default_gps, device_movement_system, 
    device_power_system, drone_trx_system, ewd_trx_system 
};


const COMMAND_CENTER_POSITION: Point3D = Point3D { x: 200.0, y: 100.0, z: 0.0 };
const DRONE_DESTINATION: Point3D       = Point3D { x: 0.0, y: 0.0, z: 0.0 };


fn attack_scenario() -> Scenario {
    let attack_task = Task::new(
        TaskType::Attack,
        Some(DRONE_DESTINATION)
    );

    Scenario::from([(0, BROADCAST_ID, attack_task)])
}

fn reposition_scenario() -> Scenario {
    let task1 = Task::new(
        TaskType::Reposition,
        Some(DRONE_DESTINATION)
    );
    let task2 = Task::new(
        TaskType::Reposition,
        Some(Point3D::new(0.0, 0.0, 150.0))
    );
    let task3 = Task::new(
        TaskType::Reposition,
        Some(Point3D::new(0.0, 150.0, 150.0))
    );
    let task4 = task1;

    Scenario::from([
        (0, BROADCAST_ID, task1),
        (250, BROADCAST_ID, task2),
        (4000, BROADCAST_ID, task3),
        (6000, BROADCAST_ID, task4),
    ])
}

fn derive_filename(
    trx_system_type: TRXSystemType, 
    topology: Topology,
    text: &str
) -> String {
    let trx_system_part = match trx_system_type {
        TRXSystemType::Color    => "col",
        TRXSystemType::Strength => "str",
    };
    let topology_part = match topology {
        Topology::Mesh => "mesh",
        Topology::Star => "star",
    };

    format!("{trx_system_part}_{text}_{topology_part}.gif")
}

pub fn indicator_malware() -> Malware {
    Malware::new(
        MalwareType::Indicator, 
        MALWARE_INFECTION_DELAY,
        MALWARE_SPREAD_DELAY
    )
}


pub fn gps_only(
    general_config: &GeneralConfig,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) {
    let cc_tx_control_area_radius    = 300.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_level    = RED_SIGNAL_LEVEL; 
    let ewd_suppression_area_radius  = 50.0; 
        
    let command_center = DeviceBuilder::new()
        .set_real_position(COMMAND_CENTER_POSITION)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().trx_system_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        drone_positions,
        vulnerabilities,
        general_config.model_config().trx_system_type(),
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_level, 
    );
    devices.insert(0, command_center);
 
    let ewd_gps = DeviceBuilder::new()
        .set_real_position(Point3D::new(0.0, 5.0, 2.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().trx_system_type(), 
                GPS_L1_FREQUENCY, 
                ewd_suppression_area_radius
            )
        )
        .build();
    let attacker_devices = vec![
        AttackerDevice::new(ewd_gps, AttackType::ElectronicWarfare)
    ];

    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(IdToDeviceMap::from(devices.as_slice()))
        .set_attacker_devices(attacker_devices)
        .set_gps(default_gps(general_config.model_config().trx_system_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();

    let output_filename = derive_filename(
        general_config.model_config().trx_system_type(), 
        general_config.model_config().topology(), 
        "gps_only"
    );
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.render_config().plot_caption(),
        general_config.render_config().plot_resolution(),
        DEFAULT_AXES_RANGE,
        DEFAULT_DEVICE_COLORING,
        DEFAULT_CAMERA_ANGLE,
    );

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn movement(
    general_config: &GeneralConfig,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) {
    let cc_tx_control_area_radius    = 300.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_level    = SignalLevel::from(10_000.0); 

    let command_center = DeviceBuilder::new()
        .set_real_position(COMMAND_CENTER_POSITION)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().trx_system_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(), 
        drone_positions,
        vulnerabilities,
        general_config.model_config().trx_system_type(),
        drone_tx_control_area_radius,
        drone_gps_rx_signal_level,
    ); 
    devices.insert(0, command_center);
    
    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(IdToDeviceMap::from(devices.as_slice()))
        .set_gps(default_gps(general_config.model_config().trx_system_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(reposition_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();

    let output_filename = derive_filename(
        general_config.model_config().trx_system_type(),
        general_config.model_config().topology(), 
        "movement"
    );
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.render_config().plot_caption(),
        general_config.render_config().plot_resolution(),
        DEFAULT_AXES_RANGE,
        DEFAULT_DEVICE_COLORING,
        DEFAULT_CAMERA_ANGLE
    );

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn gps_spoofing(
    general_config: &GeneralConfig,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) {
    let cc_tx_control_area_radius    = 300.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_level    = RED_SIGNAL_LEVEL; 
    let gps_spoofing_area_radius     = 100.0; 
        
    let command_center = DeviceBuilder::new()
        .set_real_position(COMMAND_CENTER_POSITION)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().trx_system_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        drone_positions,
        vulnerabilities,
        general_config.model_config().trx_system_type(),
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_level, 
    ); 
    devices.insert(0, command_center);

    let ewd_gps = DeviceBuilder::new()
        .set_real_position(Point3D::new(0.0, 5.0, 2.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().trx_system_type(), 
                GPS_L1_FREQUENCY, 
                gps_spoofing_area_radius
            )
        )
        .build();
    let spoofed_position = Point3D::new(-200.0, -100.0, -200.0);
    let attacker_devices = vec![
        AttackerDevice::new(ewd_gps, AttackType::GPSSpoofing(spoofed_position))
    ];

    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(IdToDeviceMap::from(devices.as_slice()))
        .set_attacker_devices(attacker_devices)
        .set_gps(default_gps(general_config.model_config().trx_system_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();

    let output_filename = derive_filename(
        general_config.model_config().trx_system_type(),
        general_config.model_config().topology(), 
        "gps_spoofing"
    );
    let axes_ranges    = Axes3DRanges::new(0.0..200.0, 0.0..0.0, 0.0..200.0);
    let camera_angle   = CameraAngle::new(1.57, 1.57);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.render_config().plot_caption(),
        general_config.render_config().plot_resolution(),
        axes_ranges,
        DEFAULT_DEVICE_COLORING,
        camera_angle,
    );

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn malware_infection(
    general_config: &GeneralConfig,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) {
    let cc_tx_control_area_radius    = 200.0;
    let drone_tx_control_area_radius = 15.0;
    let drone_gps_rx_signal_level    = GREEN_SIGNAL_LEVEL; 
    let attacker_tx_area_radius      = 50.0;
    let malware = general_config.model_config().malware()
        .expect("Missing malware type");

    let command_center = DeviceBuilder::new()
        .set_real_position(Point3D::new(100.0, 50.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().trx_system_type(), 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .set_vulnerabilities(&[malware])
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.model_config().drone_count(),
        drone_positions,
        vulnerabilities,
        general_config.model_config().trx_system_type(),
        drone_tx_control_area_radius,
        drone_gps_rx_signal_level,
    ); 
    devices.insert(0, command_center);
    
    let attacker = DeviceBuilder::new()
        .set_real_position(Point3D::new(-10.0, 2.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                general_config.model_config().trx_system_type(),
                CONTROL_FREQUENCY,
                attacker_tx_area_radius
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
        .set_device_map(IdToDeviceMap::from(devices.as_slice()))
        .set_gps(default_gps(general_config.model_config().trx_system_type()))
        .set_topology(general_config.model_config().topology())
        .set_delay_multiplier(general_config.model_config().delay_multiplier());
    
    if general_config.render_config().display_malware_propagation() {
        malware_propagation(
            attacker,
            drone_network_builder.clone(),
            general_config,
        );
    }

    let drone_network = drone_network_builder
        .set_attacker_devices(attacker_devices)
        .build();

    let text = match malware.malware_type() {
        MalwareType::DoS(_)     => "mal_dos",
        MalwareType::Indicator  => "mal_indicator",
    };
    let output_filename = derive_filename(
        general_config.model_config().trx_system_type(),
        general_config.model_config().topology(), 
        text,
    );
    let drone_coloring = match malware.malware_type() {
        MalwareType::DoS(_)    => DeviceColoring::SingleColor(0, 0, 0),
        MalwareType::Indicator => DeviceColoring::Infection,
    };
    let axes_ranges    = Axes3DRanges::new(0.0..100.0, 0.0..0.0, 0.0..100.0);
    let camera_angle   = CameraAngle::new(1.57, 1.57);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.render_config().plot_caption(),
        general_config.render_config().plot_resolution(),
        axes_ranges,
        drone_coloring,
        camera_angle
    );

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().output_directory(),
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

    let output_filename = derive_filename(
        general_config.model_config().trx_system_type(),
        general_config.model_config().topology(), 
        "mal_indicator",
    );
    let drone_coloring = DeviceColoring::Infection; 
    let axes_ranges    = Axes3DRanges::new(0.0..100.0, 0.0..0.0, 0.0..100.0);
    let camera_angle   = CameraAngle::new(1.57, 1.57);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.render_config().plot_caption(),
        general_config.render_config().plot_resolution(),
        axes_ranges,
        drone_coloring,
        camera_angle
    );

    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}

pub fn signal_loss_response(
    general_config: &GeneralConfig,
    _drone_positions: &[Point3D],
    _vulnerabilities: &[Vec<Malware>],
) {
    let cc_tx_control_area_radius    = 200.0;
    let drone_tx_control_area_radius = 50.0;
    let drone_gps_rx_signal_level    = GREEN_SIGNAL_LEVEL; 
    let control_ewd_suppression_area_radius = 25.0;
    let command_center_position      = Point3D::new(100.0, 50.0, 0.0);

    let command_center = DeviceBuilder::new()
        .set_real_position(command_center_position)
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                general_config.model_config().trx_system_type(), 
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
                general_config.model_config().trx_system_type(), 
                drone_tx_control_area_radius, 
                drone_gps_rx_signal_level
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
                general_config.model_config().trx_system_type(),
                CONTROL_FREQUENCY,
                control_ewd_suppression_area_radius
            )
        )
        .build();
    let attacker_devices = vec![
        AttackerDevice::new(ewd_control, AttackType::ElectronicWarfare)
    ];
    
    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(IdToDeviceMap::from(devices.as_slice()))
        .set_attacker_devices(attacker_devices)
        .set_gps(default_gps(general_config.model_config().trx_system_type()))
        .set_topology(general_config.model_config().topology())
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.model_config().delay_multiplier())
        .build();
 
    let output_filename = derive_filename(
        general_config.model_config().trx_system_type(),
        general_config.model_config().topology(),
        "signal_loss_response"
    ); 
    let axes_ranges     = Axes3DRanges::new(0.0..100.0, 0.0..100.0, 0.0..100.0);
    let renderer        = PlottersRenderer::new(
        &output_filename,
        general_config.render_config().plot_caption(),
        general_config.render_config().plot_resolution(),
        axes_ranges,
        DEFAULT_DEVICE_COLORING,
        DEFAULT_CAMERA_ANGLE,
    );
    
    let mut model_player = ModelPlayer::new(
        general_config.model_player_config().output_directory(),
        drone_network,
        renderer,
        general_config.model_player_config().simulation_time(),
    );

    model_player.play();
}
