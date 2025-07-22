use std::ops::Range;

use rand::prelude::*;

use crate::backend::CONTROL_FREQUENCY;
use crate::backend::device::{
    Device, DeviceBuilder, SignalLossResponse, BROADCAST_ID, MAX_DRONE_SPEED 
};
use crate::backend::device::systems::{
    MovementSystem, PowerSystem, RXModule, SecuritySystem, TRXSystem, TXModule, 
    TRXSystemType
};
use crate::backend::malware::{Malware, MalwareType};
use crate::backend::mathphysics::{Megahertz, Meter, Point3D, PowerUnit};
use crate::backend::networkmodel::gps::GPS;
use crate::backend::signal::{
    FreqToLevelMap, SignalArea, SignalLevel, GPS_L1_FREQUENCY, 
    GREEN_SIGNAL_LEVEL 
};
use crate::backend::task::{Scenario, Task, TaskType};
use crate::frontend::{MALWARE_INFECTION_DELAY, MALWARE_SPREAD_DELAY};


pub const DEVICE_MAX_POWER: PowerUnit = 100_000;
pub const NETWORK_ORIGIN: Point3D     = Point3D { x: 150.3, y: 90.6, z: 25.5 };
pub const CC_POSITION: Point3D        = Point3D { x: 200.0, y: 100.0, z: 0.0 };

const DEFAULT_GPS_POSITION_IN_METERS: Point3D = Point3D { 
    x: NETWORK_ORIGIN.x, 
    y: NETWORK_ORIGIN.y, 
    z: 200.0
};
const DRONE_DESTINATION: Point3D  = Point3D { x: 0.0, y: 0.0, z: 0.0 };
const GPS_TX_RADIUS: Meter = 350.0;
const PATCH_PROBABILITY: f64 = 0.0;


pub fn create_drone_vec(
    drone_count: usize, 
    network_position: &NetworkPosition,
    malware: Option<Malware>,
    trx_system_type: TRXSystemType,
    tx_control_area_radius: Meter,
    max_gps_rx_signal_level: SignalLevel,
) -> Vec<Device> {
    let power_system    = device_power_system();
    let movement_system = device_movement_system();
    let trx_system      = drone_trx_system(
        trx_system_type, 
        tx_control_area_radius,
        max_gps_rx_signal_level
    );
    let patches = match malware {
        Some(malware) => vec![malware],
        None          => Vec::new(),
    };
    let security_system = SecuritySystem::new(patches);

    let drone_builder = DeviceBuilder::new()
        .set_power_system(power_system)
        .set_movement_system(movement_system)
        .set_trx_system(trx_system)
        .set_signal_loss_response(SignalLossResponse::Hover);

    (0..drone_count)
        .map(|_| {
            let drone_builder = if rand::random_bool(PATCH_PROBABILITY) {
                drone_builder
                    .clone()
                    .set_security_system(security_system.clone())
            } else { 
                drone_builder.clone()
            };

            drone_builder
                .set_real_position(generate_drone_position(network_position))
                .build()
        })  
        .collect()
}

fn generate_drone_position(network_position: &NetworkPosition) -> Point3D {
    let mut rng = rand::rng();

    let random_offset = Point3D::new(
        rng.random_range(network_position.x_offset_range.clone()),
        rng.random_range(network_position.y_offset_range.clone()),
        rng.random_range(network_position.z_offset_range.clone())
    );
    
    network_position.origin + random_offset
}

pub fn cc_trx_system(
    trx_system_type: TRXSystemType, 
    tx_control_area_radius: Meter
) -> TRXSystem {
    let tx_module = tx_module(CONTROL_FREQUENCY, tx_control_area_radius);
    let rx_module = rx_module(GREEN_SIGNAL_LEVEL);

    TRXSystem::new( 
        trx_system_type,
        tx_module, 
        rx_module
    )
}

pub fn drone_trx_system(
    trx_system_type: TRXSystemType, 
    tx_control_area_radius: Meter,
    max_gps_rx_signal_level: SignalLevel
) -> TRXSystem {
    TRXSystem::new(
        trx_system_type,
        tx_module(CONTROL_FREQUENCY, tx_control_area_radius), 
        rx_module(max_gps_rx_signal_level),
    )
}
 
pub fn ewd_trx_system(
    trx_system_type: TRXSystemType,
    frequency: Megahertz,
    suppression_area_radius: Meter
) -> TRXSystem {
    TRXSystem::new( 
        trx_system_type,
        tx_module(frequency, suppression_area_radius), 
        RXModule::default()
    )
}

pub fn default_gps(trx_system_type: TRXSystemType) -> GPS {
    let trx_system = TRXSystem::new( 
        trx_system_type,
        tx_module(GPS_L1_FREQUENCY, GPS_TX_RADIUS),
        RXModule::default()
    );

    let device = DeviceBuilder::new()
        .set_real_position(DEFAULT_GPS_POSITION_IN_METERS)
        .set_signal_loss_response(SignalLossResponse::Ignore)
        .set_power_system(device_power_system())
        .set_trx_system(trx_system)
        .build();

    GPS::new(device, GPS_L1_FREQUENCY)
}

pub fn tx_module(frequency: Megahertz, tx_area_radius: Meter) -> TXModule {
    let tx_control_area = SignalArea::build(tx_area_radius).unwrap();

    let tx_signal_levels = FreqToLevelMap::from([(
        frequency,
        SignalLevel::from_area(tx_control_area, CONTROL_FREQUENCY)
    )]);

    TXModule::new(tx_signal_levels)
}

pub fn rx_module(max_gps_rx_signal_level: SignalLevel) -> RXModule {
    let max_rx_signal_levels = FreqToLevelMap::from([
        (CONTROL_FREQUENCY, SignalLevel::from(10_000.0)),
        (GPS_L1_FREQUENCY, max_gps_rx_signal_level)
    ]);

    RXModule::new(max_rx_signal_levels)
}

pub fn device_power_system() -> PowerSystem {
    PowerSystem::build(DEVICE_MAX_POWER, DEVICE_MAX_POWER)
        .unwrap_or_else(|error| panic!("{}", error))
}

pub fn device_movement_system() -> MovementSystem {
    MovementSystem::build(MAX_DRONE_SPEED)
        .unwrap_or_else(|error| panic!("{}", error))
}

pub fn default_network_position(network_origin: Point3D) -> NetworkPosition {
    NetworkPosition::new(
        network_origin,
        -40.0..40.0,
        -40.0..40.0,
        -20.0..20.0,
    )
}

pub fn attack_scenario() -> Scenario {
    let attack_task = Task::new(
        TaskType::Attack,
        Some(DRONE_DESTINATION)
    );

    Scenario::from([(0, BROADCAST_ID, attack_task)])
}

pub fn reposition_scenario() -> Scenario {
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

pub fn indicator_malware() -> Malware {
    Malware::new(
        MalwareType::Indicator, 
        MALWARE_INFECTION_DELAY,
        MALWARE_SPREAD_DELAY
    )
}


pub struct NetworkPosition {
    origin: Point3D,
    x_offset_range: Range<f32>,
    y_offset_range: Range<f32>,
    z_offset_range: Range<f32>,
}

impl NetworkPosition {
    #[must_use]
    pub fn new(
        origin: Point3D,
        x_offset_range: Range<f32>,
        y_offset_range: Range<f32>,
        z_offset_range: Range<f32>,
    ) -> Self {
        Self { 
            origin, 
            x_offset_range,
            y_offset_range,
            z_offset_range
        }
    }
}
