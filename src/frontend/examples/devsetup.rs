use std::ops::Range;

use rand::prelude::*;

use crate::backend::CONTROL_FREQUENCY;
use crate::backend::device::{
    Device, DeviceBuilder, SignalLossResponse, MAX_DRONE_SPEED 
};
use crate::backend::device::systems::{
    MovementSystem, PowerSystem, RXModule, TRXSystem, TXModule, TRXSystemType
};
use crate::backend::malware::Malware;
use crate::backend::mathphysics::{Megahertz, Meter, Point3D, PowerUnit};
use crate::backend::networkmodel::gps::GPS;
use crate::backend::signal::{
    FreqToLevelMap, SignalArea, SignalLevel, GPS_L1_FREQUENCY, 
    GREEN_SIGNAL_LEVEL 
};


pub const DEVICE_MAX_POWER: PowerUnit = 100_000;
pub const NETWORK_ORIGIN: Point3D     = Point3D { x: 150.3, y: 90.6, z: 25.5 };


const VULNERABILITY_PROBABILITY: f64 = 1.0;

const GPS_TX_RADIUS: Meter = 350.0;
const DEFAULT_GPS_POSITION_IN_METERS: Point3D = Point3D { 
    x: NETWORK_ORIGIN.x, 
    y: NETWORK_ORIGIN.y, 
    z: 200.0
};


pub fn generate_drone_positions(
    drone_count: usize,
    network_position: &NetworkPosition,
) -> Vec<Point3D> {
    let mut rng = rand::rng();

    (1..=drone_count)
        .map(|_| {
            let random_offset = Point3D::new(
                rng.random_range(network_position.x_offset_range.clone()),
                rng.random_range(network_position.y_offset_range.clone()),
                rng.random_range(network_position.z_offset_range.clone())
            );
            
            network_position.origin + random_offset
        })
        .collect()
}

pub fn generate_drone_vulnerabilities(
    drone_count: usize,
    vulnerabilities: &[Malware],
) -> Vec<Vec<Malware>> {
    (1..=drone_count)
        .map(|_| {
            if rand::random_bool(VULNERABILITY_PROBABILITY) {
                Vec::from(vulnerabilities)
            } else {
                Vec::new()
            }
        })
        .collect()
}

pub fn create_drone_vec(
    drone_count: usize, 
    drone_positions: &[Point3D],
    vulnerabilities: &[Vec<Malware>],
    trx_system_type: TRXSystemType,
    tx_control_area_radius: Meter,
    max_gps_rx_signal_level: SignalLevel,
) -> Vec<Device> {
    assert_eq!(drone_count, drone_positions.len());
    assert_eq!(drone_count, vulnerabilities.len());

    let power_system    = device_power_system();
    let movement_system = device_movement_system();
    let trx_system      = drone_trx_system(
        trx_system_type, 
        tx_control_area_radius,
        max_gps_rx_signal_level
    );

    let drone_builder = DeviceBuilder::new()
        .set_power_system(power_system)
        .set_movement_system(movement_system)
        .set_trx_system(trx_system)
        .set_signal_loss_response(SignalLossResponse::Hover);

    (0..drone_count)
        .map(|i| {
            let drone_builder = drone_builder.clone();

            drone_builder
                .set_real_position(drone_positions[i])
                .set_vulnerabilities(&vulnerabilities[i])
                .build()
        })  
        .collect()
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
