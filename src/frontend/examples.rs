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

use super::{MALWARE_INFECTION_DELAY, MALWARE_SPREAD_DELAY};
use super::cli::GeneralConfig;
use super::player::ModelPlayer;
use super::renderer::{
    Axes3DRanges, CameraAngle, DeviceColoring, PlottersRenderer
};

pub use devsetup::DEVICE_MAX_POWER;

use devsetup::{
    NETWORK_ORIGIN, NetworkPosition, cc_trx_system, create_drone_vec, 
    default_gps, device_movement_system, device_power_system, drone_trx_system, 
    ewd_trx_system, generate_drone_positions, generate_drone_vulnerabilities
};


mod devsetup;


const DRONE_DESTINATION: Point3D       = Point3D { x: 0.0, y: 0.0, z: 0.0 };
const COMMAND_CENTER_POSITION: Point3D = Point3D { x: 200.0, y: 100.0, z: 0.0 };


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

fn indicator_malware() -> Malware {
    Malware::new(
        MalwareType::Indicator, 
        MALWARE_INFECTION_DELAY,
        MALWARE_SPREAD_DELAY
    )
}


#[derive(Clone, Copy)]
pub enum Example {
    GPSEWD,
    GPSSpoofing,
    MalwareInfection,
    Movement,
    SignalLossResponse,
}

impl Example {
    pub fn execute(&self, general_config: &GeneralConfig) {
        let network_origin = match self {
            Example::MalwareInfection => Point3D::new(50.0, 50.0, 0.0),
            _                         => NETWORK_ORIGIN,
        };

        let network_position = NetworkPosition::new(
            network_origin,
            -40.0..40.0,
            -40.0..40.0,
            -20.0..20.0,
        );
        
        let drone_positions = generate_drone_positions(
            general_config.drone_count(), 
            &network_position
        );
        let malware_list = match general_config.malware() {
            Some(malware) if general_config.display_malware_propagation() =>
                vec![malware, indicator_malware()],
            Some(malware) => vec![malware],
            None          => Vec::new()
        };
        let vulnerabilities = generate_drone_vulnerabilities(
            general_config.drone_count(), 
            &malware_list
        );

        let example_function = match self {
            Self::GPSEWD             => gps_only,
            Self::GPSSpoofing        => gps_spoofing,
            Self::MalwareInfection   => malware_infection,
            Self::Movement           => movement,
            Self::SignalLossResponse => signal_loss_response,
        };

        execute_example_function_by_general_config(
            example_function,
            general_config,
            &drone_positions,
            &vulnerabilities,
        ); 
    }
}


fn execute_example_function_by_general_config<F>(
    example_function: F,
    general_config: &GeneralConfig,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) 
where
    F: Fn(&GeneralConfig, TRXSystemType, Topology, &[Point3D], &[Vec<Malware>])
{
    match (general_config.trx_system_type(), general_config.topology()) {
        (Some(trx_system_type), Some(topology)) => 
            example_function(
                general_config, 
                trx_system_type,
                topology,
                drone_positions, 
                vulnerabilities
            ),
        (Some(trx_system_type), None) => 
            execute_example_function_with_all_topologies(
                example_function, 
                general_config, 
                trx_system_type, 
                drone_positions, 
                vulnerabilities
            ),
        (None, Some(topology)) => 
            execute_example_function_with_all_trx_system_types(
                example_function, 
                general_config, 
                topology, 
                drone_positions, 
                vulnerabilities
            ),
        (None, None) =>
            execute_example_function_with_all_trx_system_types_and_topologies(
                example_function, 
                general_config, 
                drone_positions, 
                vulnerabilities
            ),
    }
}

fn execute_example_function_with_all_trx_system_types<F>(
    example_function: F,
    general_config: &GeneralConfig,
    topology: Topology,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) 
where
    F: Fn(&GeneralConfig, TRXSystemType, Topology, &[Point3D], &[Vec<Malware>])
{
    example_function(
        general_config, 
        TRXSystemType::Color, 
        topology,
        drone_positions, 
        vulnerabilities
    );
    example_function(
        general_config, 
        TRXSystemType::Strength, 
        topology,
        drone_positions, 
        vulnerabilities
    );
}

fn execute_example_function_with_all_topologies<F>(
    example_function: F,
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) 
where
    F: Fn(&GeneralConfig, TRXSystemType, Topology, &[Point3D], &[Vec<Malware>])
{
    example_function(
        general_config, 
        trx_system_type,
        Topology::Star,
        drone_positions, 
        vulnerabilities
    );
    example_function(
        general_config, 
        trx_system_type,
        Topology::Mesh,
        drone_positions, 
        vulnerabilities
    );
}

fn execute_example_function_with_all_trx_system_types_and_topologies<F>(
    example_function: F,
    general_config: &GeneralConfig,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
)
where
    F: Fn(&GeneralConfig, TRXSystemType, Topology, &[Point3D], &[Vec<Malware>])
{
    execute_example_function_with_all_trx_system_types(
        &example_function, 
        general_config, 
        Topology::Star, 
        drone_positions, 
        vulnerabilities
    );
    execute_example_function_with_all_trx_system_types(
        example_function, 
        general_config, 
        Topology::Mesh, 
        drone_positions, 
        vulnerabilities
    );
}

fn gps_only(
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    topology: Topology,
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
                trx_system_type, 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.drone_count(),
        drone_positions,
        vulnerabilities,
        trx_system_type,
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_level, 
    );
    devices.insert(0, command_center);
 
    let ewd_gps = DeviceBuilder::new()
        .set_real_position(Point3D::new(0.0, 5.0, 2.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                trx_system_type, 
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
        .set_gps(default_gps(trx_system_type))
        .set_topology(topology)
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.delay_multiplier())
        .build();

    let output_filename = derive_filename(
        trx_system_type, 
        topology, 
        "gps_only"
    );
    let drone_coloring = DeviceColoring::SingleColor(0, 0, 0); 
    let camera_angle   = CameraAngle::new(0.15, 0.5);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.plot_caption(),
        general_config.plot_resolution(),
        Axes3DRanges::default(),
        drone_coloring,
        camera_angle
    );

    let mut model_player = ModelPlayer::new(
        general_config.output_directory(),
        drone_network,
        renderer,
        general_config.simulation_time(),
    );

    model_player.play();
}

fn movement(
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    topology: Topology,
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
                trx_system_type, 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.drone_count(), 
        drone_positions,
        vulnerabilities,
        trx_system_type,
        drone_tx_control_area_radius,
        drone_gps_rx_signal_level,
    ); 
    devices.insert(0, command_center);
    
    let drone_network = NetworkModelBuilder::new()
        .set_command_center_id(command_center_id)
        .set_device_map(IdToDeviceMap::from(devices.as_slice()))
        .set_gps(default_gps(trx_system_type))
        .set_topology(topology)
        .set_scenario(reposition_scenario())
        .set_delay_multiplier(general_config.delay_multiplier())
        .build();

    let output_filename = derive_filename(
        trx_system_type,
        topology, 
        "movement"
    );
    let drone_coloring = DeviceColoring::SingleColor(0, 0, 0);
    let camera_angle   = CameraAngle::new(0.15, 0.5);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.plot_caption(),
        general_config.plot_resolution(),
        Axes3DRanges::default(),
        drone_coloring,
        camera_angle
    );

    let mut model_player = ModelPlayer::new(
        general_config.output_directory(),
        drone_network,
        renderer,
        general_config.simulation_time(),
    );

    model_player.play();
}

fn gps_spoofing(
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    topology: Topology,
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
                trx_system_type, 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.drone_count(),
        drone_positions,
        vulnerabilities,
        trx_system_type,
        drone_tx_control_area_radius, 
        drone_gps_rx_signal_level, 
    ); 
    devices.insert(0, command_center);

    let ewd_gps = DeviceBuilder::new()
        .set_real_position(Point3D::new(0.0, 5.0, 2.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                trx_system_type, 
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
        .set_gps(default_gps(trx_system_type))
        .set_topology(topology)
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.delay_multiplier())
        .build();

    let output_filename = derive_filename(
        trx_system_type,
        topology, 
        "gps_spoofing"
    );
    let axes_ranges    = Axes3DRanges::new(0.0..200.0, 0.0..0.0, 0.0..200.0);
    let drone_coloring = DeviceColoring::SingleColor(0, 0, 0); 
    let camera_angle   = CameraAngle::new(1.57, 1.57);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.plot_caption(),
        general_config.plot_resolution(),
        axes_ranges,
        drone_coloring,
        camera_angle,
    );

    let mut model_player = ModelPlayer::new(
        general_config.output_directory(),
        drone_network,
        renderer,
        general_config.simulation_time(),
    );

    model_player.play();
}

fn malware_infection(
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    topology: Topology,
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
) {
    let cc_tx_control_area_radius    = 200.0;
    let drone_tx_control_area_radius = 15.0;
    let drone_gps_rx_signal_level    = GREEN_SIGNAL_LEVEL; 
    let attacker_tx_area_radius      = 50.0;
    let malware = general_config.malware()
        .expect("Missing malware type");

    let command_center = DeviceBuilder::new()
        .set_real_position(Point3D::new(100.0, 50.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            cc_trx_system(
                trx_system_type, 
                cc_tx_control_area_radius
            )
        )
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .set_vulnerabilities(&[malware])
        .build();
    let command_center_id = command_center.id();

    let mut devices = create_drone_vec(
        general_config.drone_count(),
        drone_positions,
        vulnerabilities,
        trx_system_type,
        drone_tx_control_area_radius,
        drone_gps_rx_signal_level,
    ); 
    devices.insert(0, command_center);
    
    let attacker = DeviceBuilder::new()
        .set_real_position(Point3D::new(-10.0, 2.0, 0.0))
        .set_power_system(device_power_system())
        .set_trx_system(
            ewd_trx_system(
                trx_system_type,
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
        .set_gps(default_gps(trx_system_type))
        .set_topology(topology)
        .set_delay_multiplier(general_config.delay_multiplier());
    
    if general_config.display_malware_propagation() {
        malware_propagation(
            attacker,
            drone_network_builder.clone(),
            general_config,
            trx_system_type,
            topology,
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
        trx_system_type,
        topology, 
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
        general_config.plot_caption(),
        general_config.plot_resolution(),
        axes_ranges,
        drone_coloring,
        camera_angle
    );

    let mut model_player = ModelPlayer::new(
        general_config.output_directory(),
        drone_network,
        renderer,
        general_config.simulation_time(),
    );

    model_player.play();
}

fn malware_propagation(
    attacker: Device,
    drone_network_builder: NetworkModelBuilder,
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    topology: Topology,
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
        trx_system_type,
        topology, 
        "mal_indicator",
    );
    let drone_coloring = DeviceColoring::Infection; 
    let axes_ranges    = Axes3DRanges::new(0.0..100.0, 0.0..0.0, 0.0..100.0);
    let camera_angle   = CameraAngle::new(1.57, 1.57);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.plot_caption(),
        general_config.plot_resolution(),
        axes_ranges,
        drone_coloring,
        camera_angle
    );

    let mut model_player = ModelPlayer::new(
        general_config.output_directory(),
        drone_network,
        renderer,
        general_config.simulation_time(),
    );

    model_player.play();
}

fn signal_loss_response(
    general_config: &GeneralConfig,
    trx_system_type: TRXSystemType,
    topology: Topology,
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
                trx_system_type, 
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
                trx_system_type, 
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
                trx_system_type,
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
        .set_gps(default_gps(trx_system_type))
        .set_topology(topology)
        .set_scenario(attack_scenario())
        .set_delay_multiplier(general_config.delay_multiplier())
        .build();
 
    let output_filename = derive_filename(
        trx_system_type,
        topology,
        "signal_loss_response"
    ); 
    let axes_ranges    = Axes3DRanges::new(0.0..100.0, 0.0..100.0, 0.0..100.0);
    let drone_coloring = DeviceColoring::SingleColor(0, 0, 0); 
    let camera_angle   = CameraAngle::new(0.15, 0.5);
    let renderer       = PlottersRenderer::new(
        &output_filename,
        general_config.plot_caption(),
        general_config.plot_resolution(),
        axes_ranges,
        drone_coloring,
        camera_angle,
    );
    
    let mut model_player = ModelPlayer::new(
        general_config.output_directory(),
        drone_network,
        renderer,
        general_config.simulation_time(),
    );

    model_player.play();
}
