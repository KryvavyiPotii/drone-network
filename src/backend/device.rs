use std::sync::atomic::{AtomicUsize, Ordering};

use log::info;
use serde::Serialize;
use thiserror::Error;

use super::{CONTROL_FREQUENCY, DESTINATION_RADIUS, ITERATION_TIME};
use super::malware::{InfectionState, Malware, MalwareToStateMap, MalwareType};
use super::mathphysics::{
    equation_of_motion_3d, millis_to_secs, Megahertz, Meter, MeterPerSecond, 
    Millisecond, Point3D, Position, PowerUnit
};
use super::signal::{
    Data, FreqToLevelMap, Signal, SignalArea, SignalLevel, BLACK_SIGNAL_LEVEL, 
    GPS_L1_FREQUENCY
};
use super::task::{Task, TaskType};

use systems::{
    MovementSystem, PowerSystem, PowerSystemError, TRXSystem, TRXSystemError
};


pub use idmaps::*;


pub mod idmaps;
pub mod systems;


pub type DeviceId = usize;


pub const BROADCAST_ID: DeviceId = 0;

pub const MAX_DRONE_SPEED: MeterPerSecond = 25.0;


const MOVEMENT_POWER_CONSUMPTION: PowerUnit   = 5; 
const PASSIVE_POWER_CONSUMPTION: PowerUnit    = 1; 
const PROCESSING_POWER_CONSUMPTION: PowerUnit = 5; 


static FREE_DEVICE_ID: AtomicUsize = AtomicUsize::new(1);


fn generate_device_id() -> DeviceId {
    FREE_DEVICE_ID.fetch_add(1, Ordering::SeqCst)
}


#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("System is disabled")]
    DisabledSystem,
    #[error("Power system failed with error `{0}`")]
    PowerSystemError(#[from] PowerSystemError),
    #[error("TRX system failed with error `{0}`")]
    TRXSystemError(#[from] TRXSystemError),
}


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub enum SignalLossResponse {
    Ascend,
    #[default]
    Ignore,
    Hover,
    ReturnToHome(Point3D), // Point3D is a home point
    Shutdown,
}


#[derive(Clone, Debug)]
pub struct DeviceBuilder {
    real_position_in_meters: Option<Point3D>,
    task: Option<Task>,
    power_system: Option<PowerSystem>,
    movement_system: Option<MovementSystem>,
    trx_system: Option<TRXSystem>,
    vulnerabilities: Option<Vec<Malware>>,
    signal_loss_response: Option<SignalLossResponse>,
}

impl DeviceBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            real_position_in_meters: None,
            task: None,
            power_system: None,
            movement_system: None,
            trx_system: None,
            vulnerabilities: None,
            signal_loss_response: None,
        }
    }

    #[must_use]
    pub fn set_real_position(
        mut self, 
        real_position_in_meters: Point3D
    ) -> Self {
        self.real_position_in_meters = Some(real_position_in_meters);
        self
    }
    
    #[must_use]
    pub fn set_task(mut self, task: Task) -> Self {
        self.task = Some(task);
        self
    }
    
    #[must_use]
    pub fn set_power_system(mut self, power_system: PowerSystem) -> Self {
        self.power_system = Some(power_system);
        self
    }
    
    #[must_use]
    pub fn set_movement_system(
        mut self, 
        movement_system: MovementSystem
    ) -> Self {
        self.movement_system = Some(movement_system);
        self
    }
    
    #[must_use]
    pub fn set_trx_system(mut self, trx_system: TRXSystem) -> Self {
        self.trx_system = Some(trx_system);
        self
    }

    #[must_use]
    pub fn set_vulnerabilities(
        mut self, 
        vulnerabilities: &[Malware]
    ) -> Self {
        self.vulnerabilities = Some(vulnerabilities.to_vec());
        self
    }

    #[must_use]
    pub fn set_signal_loss_response(
        mut self,
        signal_loss_response: SignalLossResponse
    ) -> Self {
        self.signal_loss_response = Some(signal_loss_response);
        self
    }
   
    #[must_use]
    pub fn build(self) -> Device {
        Device::new(
            generate_device_id(),
            self.real_position_in_meters.unwrap_or_default(),
            self.task.unwrap_or_default(),
            self.power_system.unwrap_or_default(),
            self.movement_system.unwrap_or_default(),
            self.trx_system.unwrap_or_default(),
            self.vulnerabilities.unwrap_or_default().as_ref(),
            self.signal_loss_response.unwrap_or_default(),
        )
    }
}

impl Default for DeviceBuilder {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Clone, Debug, Serialize)]
pub struct Device {
    id: DeviceId,
    current_time: Millisecond,
    real_position_in_meters: Point3D,
    task: Task,
    power_system: PowerSystem,
    movement_system: MovementSystem,
    trx_system: TRXSystem,
    infection_states: MalwareToStateMap,
    signal_loss_response: SignalLossResponse,
}

impl Device {
    #[must_use]
    pub fn new(
        id: DeviceId,
        real_position_in_meters: Point3D,
        task: Task,
        power_system: PowerSystem,
        movement_system: MovementSystem,
        trx_system: TRXSystem,
        vulnerabilities: &[Malware],
        signal_loss_response: SignalLossResponse,
    ) -> Self {
        let infection_states = vulnerabilities
            .iter()
            .map(|malware| (*malware, (0, InfectionState::Vulnerable)))
            .collect();

        Self {
            id,
            current_time: 0,
            real_position_in_meters,
            task,
            power_system,
            movement_system,
            trx_system,
            infection_states,
            signal_loss_response,
        }
    }

    #[must_use]
    pub fn id(&self) -> DeviceId {
        self.id
    }
    
    #[must_use]
    pub fn task(&self) -> &Task {
        &self.task
    }
    
    #[must_use]
    pub fn gps_position(&self) -> &Point3D {
        self.movement_system.position()
    }
    
    #[must_use]
    pub fn signal_loss_response(&self) -> &SignalLossResponse {
        &self.signal_loss_response
    }

    #[must_use]
    pub fn tx_signal_levels(&self) -> &FreqToLevelMap {
        self.trx_system.tx_signal_levels()
    }
    
    #[must_use]
    pub fn tx_signal_level_on(&self, frequency: &Megahertz) -> &SignalLevel {
        self.trx_system.tx_signal_level_on(frequency)
    }
    
    #[must_use]
    pub fn area_on(&self, frequency: Megahertz) -> SignalArea {
        self.trx_system.area_on(frequency)
    }

    #[must_use]
    pub fn transmits_at_position<P>(
        &self,
        position: &P,
        frequency: Megahertz,
    ) -> bool
    where
        P: Position
    {
        self.transmits_at(self.distance_to(position), frequency)
    }
    
    #[must_use]
    pub fn transmits_at(
        &self, 
        distance: Meter, 
        frequency: Megahertz
    ) -> bool {
        self.trx_system.transmits_at(distance, frequency)
    }

    #[must_use]
    pub fn infection_states(&self) -> &MalwareToStateMap {
        &self.infection_states
    }   

    #[must_use]
    pub fn is_infected(&self) -> bool {
        self.infection_states
            .values()
            .any(|infection_state| 
                matches!(infection_state, (_, InfectionState::Infected))
            )
    }
    
    #[must_use]
    pub fn is_infected_with(&self, malware: &Malware) -> bool {
        matches!(
            self.infection_states.get(malware),
            Some((_, InfectionState::Infected)),
        )
    }

    #[must_use]
    pub fn malware_infections(&self) -> Vec<(Millisecond, Malware)> {
        self.infection_states
            .iter()
            .filter_map(|(malware, (time, infection_state))| 
                match infection_state {
                    InfectionState::Infected => Some((*time, *malware)),
                    _ => None
                }
            )
            .collect()
    }
    
    #[must_use]
    pub fn is_shut_down(&self) -> bool {
        self.power_system.power() == 0
    }
    
    #[must_use]
    pub fn tx_signal_level_at<P: Position>(
        &self,
        receiver: &P,
        frequency: Megahertz
    ) -> Option<SignalLevel> {
        let distance_to_rx = self.distance_to(receiver);

        self.trx_system.tx_signal_level_at(
            distance_to_rx, 
            frequency
        )
    }

    /// # Errors
    ///
    /// Will return `Err` if receiver device is out of range.
    pub fn create_signal_for(
        &self,
        receiver: &Self,
        data: Option<Data>,
        frequency: Megahertz,
    ) -> Result<Signal, TRXSystemError> {
        let Some(signal_level) = self.tx_signal_level_at(
            receiver, 
            frequency
        ) else {
            return Err(TRXSystemError::RXOutOfRange);
        };

        let signal = Signal::new(
            self.id, 
            receiver.id(),
            data,
            frequency, 
            signal_level,
        );

        self.info_created_signal_for(&receiver.id());

        Ok(signal)
    }
    
    #[must_use]
    pub fn receives_signal_on(&self, frequency: &Megahertz) -> bool {
        self.trx_system.receives_signal_on(frequency)
    }
    
    /// # Errors
    ///
    /// Will return `Err` if signal destination ID is wrong or `TRXSystem` 
    /// failed to receive signal.
    pub fn receive_signal(
        &mut self, 
        signal: Signal,
        time: Millisecond
    ) -> Result<(), TRXSystemError> {
        let destination_id = signal.destination_id();

        if destination_id != BROADCAST_ID && self.id() != destination_id {
            return Err(TRXSystemError::WrongSignalDestination);
        }

        self.trx_system
            .receive_signal(signal, time)
            .inspect(|_| 
                info!(
                    "Current time: {}, Id: {}, Received message",
                    self.current_time,
                    self.id
                )
            )
    }

    /// # Errors
    ///
    /// Will return `Err` if all power is consumed or the movement system is
    /// disabled.
    pub fn update(&mut self) -> Result<(), DeviceError> {
        self.info_control_signal_level();

        self.try_consume_power(PASSIVE_POWER_CONSUMPTION)?;
        self.handle_malware_infections();
        self.process_received_signals()?;
        if self.receives_signal_on(&CONTROL_FREQUENCY) {
            self.process_task();
        } else {
            self.handle_signal_loss();
        }
        self.trx_system.clear_received_signals();
        self.update_real_position()?;

        self.current_time += ITERATION_TIME;

        Ok(())
    }
    
    fn process_received_signals(&mut self,) -> Result<(), DeviceError> {
        for (_, signal) in self.trx_system.received_signals() {
            if let Some(data) = signal.data() {
                self.try_consume_power(PROCESSING_POWER_CONSUMPTION)?;
                self.process_data(data); 
            }
        }

        Ok(())
    }
     
    fn process_data(&mut self, data: &Data) {
        match data {
            Data::GPS(gps_position) => self.movement_system.set_position(
                *gps_position
            ),
            Data::Malware(malware)  => self.process_malware(malware),
            Data::SetTask(task)     => self.task = *task,
        }
    }

    fn process_malware(&mut self, malware: &Malware) {
        if matches!(
            self.infection_state_on(malware), 
            InfectionState::Vulnerable
        ) {
            self.infection_states.insert(
                *malware, 
                (self.current_time, InfectionState::Infected)
            );
            self.info_infected();
        }
    }
    
    fn infection_state_on(
        &self,
        malware: &Malware
    ) -> &InfectionState {
        &self.infection_states
            .get(malware)
            .unwrap_or(&(0, InfectionState::Patched))
            .1
    }
   
    fn try_consume_power(
        &mut self, 
        power: PowerUnit
    ) -> Result<(), PowerSystemError> {
        self.power_system
            .consume_power(power)
            .inspect_err(|_| self.selfdestruction())
    }

    fn process_task(&mut self) {
        let gps_is_connected = self.receives_signal_on(&GPS_L1_FREQUENCY); 

        match self.task.destination() {
            Some(destination) if gps_is_connected  => {
                self.movement_system.set_direction(*destination);
                self.try_complete_task();
            },
            Some(_) => {
                self.set_horizontal_velocity();
            },
            _ => ()
        }
    }
    
    fn set_horizontal_velocity(&mut self) {
        let mut velocity = *self.movement_system.velocity();

        velocity.initial_point.z = 0.0;
        velocity.terminal_point.z = 0.0;
        velocity.scale_to(self.movement_system.max_speed());

        self.movement_system.set_velocity(velocity);
    }

    fn handle_signal_loss(&mut self) {
        match self.signal_loss_response {
            SignalLossResponse::Ascend                   => {
                let mut upward_point = self.real_position_in_meters;
                upward_point.z += 1.0;

                self.movement_system.set_direction(upward_point);
                self.task = Task::new(
                    TaskType::Reconnect, 
                    Some(upward_point)
                );
            },
            SignalLossResponse::Hover                    => {
                self.task = Task::new(
                    TaskType::Reconnect, 
                    Some(self.real_position_in_meters)
                );
                self.process_task();
            },
            SignalLossResponse::Ignore                   =>
                self.process_task(),
            SignalLossResponse::ReturnToHome(home_point) => {
                self.task = Task::new(
                    TaskType::Reconnect,
                    Some(home_point)
                );
                self.process_task();
            },
            SignalLossResponse::Shutdown                 =>
                self.selfdestruction(),
        }
    }

    fn update_real_position(&mut self) -> Result<(), DeviceError> {
        if self.movement_system.is_disabled() {
            return Err(DeviceError::DisabledSystem);
        }

        self.try_consume_power(MOVEMENT_POWER_CONSUMPTION)?;
        
        self.real_position_in_meters = equation_of_motion_3d(
            &self.real_position_in_meters,
            &self.movement_system.velocity().displacement(),
            millis_to_secs(ITERATION_TIME),
        );
        
        Ok(())
    }

    // Device can check if it has reached the task only if it knows
    // its current position (if it has GPS connection).
    fn try_complete_task(&mut self) {
        let Some(destination) = self.task.destination() else {
            return;
        };

        match self.task.task_type() {
            TaskType::Attack if self.at_destination(destination)     => { 
                self.info_reached_destination();
                self.selfdestruction();
            },
            TaskType::Reposition if self.at_destination(destination) => { 
                self.info_reached_destination();
                self.task = Task::default();
            },
            _ => (),
        }
    }

    #[must_use]
    pub fn at_destination(&self, destination: &Point3D) -> bool {
        self.distance_to(destination) <= DESTINATION_RADIUS 
    }

    fn selfdestruction(&mut self) {
        self.power_system    = PowerSystem::default();
        self.movement_system = MovementSystem::default();
        self.trx_system      = TRXSystem::default();
    }

    fn handle_malware_infections(&mut self) {
        for (infection_time, malware) in &self.malware_infections() {
            if self.current_time != infection_time + malware.infection_delay() {
                continue;
            }

            match malware.malware_type() {
                MalwareType::DoS(lost_power) => {
                    let _ = self.try_consume_power(*lost_power);
                },
                MalwareType::Indicator       => (),
            }
        }
    }


    fn info_control_signal_level(&self) {
        info!(
            "Current time: {}, Id: {}, Control signal level: {}",
            self.current_time,
            self.id,
            self.trx_system
                .received_signal_on(&CONTROL_FREQUENCY)
                .map_or(BLACK_SIGNAL_LEVEL, |(_, signal)| *signal.level())
        );
    }

    fn info_created_signal_for(&self, receiver_id: &DeviceId) {
        info!(
            "Current time: {}, Id: {}, Created signal for {}",
            self.current_time,
            self.id,
            receiver_id
        );
    }

    fn info_infected(&self) {
        info!(
            "Current time: {}, Id: {}, Device was infected",
            self.current_time,
            self.id,
        );
    }

    fn info_reached_destination(&self) {
        info!(
            "Current time: {}, Id: {}, Reached destination",
            self.current_time,
            self.id,
        );
    }
}

impl Default for Device {
    fn default() -> Self {
        Self {
            id: generate_device_id(),
            current_time: 0,
            real_position_in_meters: Point3D::default(),
            task: Task::default(),
            power_system: PowerSystem::default(),
            movement_system: MovementSystem::default(),
            trx_system: TRXSystem::default(),
            infection_states: MalwareToStateMap::default(),
            signal_loss_response: SignalLossResponse::default(),
        }
    }
}

impl Position for Device {
    fn position(&self) -> &Point3D {
        &self.real_position_in_meters
    }
}


#[cfg(test)]
mod tests {
    use systems::TRXSystemType;

    use crate::backend::device::systems::{RXModule, TXModule};
    use crate::backend::signal::{GREEN_SIGNAL_LEVEL, RED_SIGNAL_LEVEL};

    use super::*;


    const DRONE_TX_CONTROL_RADIUS: Meter = 10.0;
    const DEVICE_MAX_POWER: PowerUnit    = 10_000;
    const SOME_DEVICE_ID: DeviceId       = 5;
    

    fn device_power_system() -> PowerSystem {
        PowerSystem::build(DEVICE_MAX_POWER, DEVICE_MAX_POWER)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn control_tx_module(radius: Meter) -> TXModule {
        let tx_signal_level  = SignalLevel::from_area(
            SignalArea::build(radius).unwrap(), 
            CONTROL_FREQUENCY
        );
        let tx_signal_levels = FreqToLevelMap::from([
            (CONTROL_FREQUENCY, tx_signal_level)
        ]);

        TXModule::new(tx_signal_levels)
    }

    fn rx_module() -> RXModule {
        let max_rx_signal_levels = FreqToLevelMap::from([
            (GPS_L1_FREQUENCY, GREEN_SIGNAL_LEVEL),
            (CONTROL_FREQUENCY, GREEN_SIGNAL_LEVEL)
        ]);

        RXModule::new(max_rx_signal_levels)
    }
     
    fn drone_movement_system() -> MovementSystem {
        MovementSystem::build(MAX_DRONE_SPEED)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn drone_green_trx_system() -> TRXSystem {
        TRXSystem::new(
            TRXSystemType::Strength,
            control_tx_module(DRONE_TX_CONTROL_RADIUS),
            rx_module()
        )
    }

    fn indicator_malware() -> Malware {
        Malware::new(MalwareType::Indicator, 0, None)
    }


    #[test]
    fn unique_device_ids() {
        let shared_device_builder = DeviceBuilder::new();

        let command_center = shared_device_builder
            .clone()
            .build();
        let electronic_warfare = shared_device_builder.build();
        let drone = DeviceBuilder::new().build();

        assert_ne!(command_center.id(), electronic_warfare.id());
        assert_ne!(command_center.id(), drone.id());
        assert_ne!(electronic_warfare.id(), drone.id());
    }

    #[test]
    fn same_device_ids_on_clone() {
        let device = DeviceBuilder::new().build();
        let cloned_device = device.clone();

        assert_eq!(device.id(), cloned_device.id())
    }

    #[test]
    fn device_selfdestructs_after_consuming_all_power() {
        let task  = Task::new(
            TaskType::Attack,
            Some(Point3D::new(5.0, 5.0, 5.0))
        );
        let power = PASSIVE_POWER_CONSUMPTION + MOVEMENT_POWER_CONSUMPTION;
        
        let power_system    = PowerSystem::build(power, power)
            .unwrap_or_else(|error| panic!("{}", error));
        let movement_system = MovementSystem::build(25.0)
            .unwrap_or_else(|error| panic!("{}", error));
        let trx_system      = drone_green_trx_system();

        let mut device = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(power_system.clone())
            .set_movement_system(movement_system.clone())
            .set_trx_system(trx_system.clone())
            .build();

        assert_eq!(device.task, task);
        assert_eq!(device.power_system, power_system);
        assert_eq!(device.trx_system, trx_system);

        assert!(!device.is_shut_down());
        
        assert!(
            matches!(
                device.update(), 
                Err(
                    DeviceError::PowerSystemError(
                        PowerSystemError::NoPowerLeft
                    )
                )
            )
        );
        assert!(device.is_shut_down());
    }

    #[test]
    fn ascending_on_signal_loss() {
        let signal_loss_response = SignalLossResponse::Ascend;
        let destination_point = Point3D::new(5.0, 5.0, 5.0);
        let task = Task::new(
            TaskType::Reposition,
            Some(destination_point)
        );
        
        let mut device_without_signal = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(device_power_system())
            .set_movement_system(drone_movement_system())
            .set_trx_system(drone_green_trx_system())
            .set_signal_loss_response(signal_loss_response)
            .build();
        let original_position = device_without_signal.real_position_in_meters;

        let many_iterations = ITERATION_TIME * 10;
        for time in (0..many_iterations).step_by(ITERATION_TIME as usize) {
            let gps_data = Data::GPS(*device_without_signal.position());
            let gps_signal = Signal::new(
                SOME_DEVICE_ID,
                device_without_signal.id(),
                Some(gps_data), 
                GPS_L1_FREQUENCY,
                RED_SIGNAL_LEVEL,
            );

            let _ = device_without_signal.receive_signal(gps_signal, time);
            let _ = device_without_signal.update();
        }

        assert_eq!(
            device_without_signal.real_position_in_meters.x,
            original_position.x
        );
        assert_eq!(
            device_without_signal.real_position_in_meters.y,
            original_position.y
        );
        assert!(device_without_signal.real_position_in_meters.z > 0.0);
    }
    
    #[test]
    fn hovering_on_signal_loss() {
        let signal_loss_response = SignalLossResponse::Hover;
        let destination_point = Point3D::new(5.0, 5.0, 5.0);
        let task = Task::new(
            TaskType::Reposition,
            Some(destination_point)
        );
        
        let mut device_without_signal = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(device_power_system())
            .set_movement_system(drone_movement_system())
            .set_trx_system(drone_green_trx_system())
            .set_signal_loss_response(signal_loss_response)
            .build();
        let original_position = device_without_signal.real_position_in_meters;

        let many_iterations = ITERATION_TIME * 500;
        for time in (0..many_iterations).step_by(ITERATION_TIME as usize) {
            let gps_data = Data::GPS(*device_without_signal.position());
            let gps_signal = Signal::new(
                SOME_DEVICE_ID,
                device_without_signal.id(),
                Some(gps_data), 
                GPS_L1_FREQUENCY,
                RED_SIGNAL_LEVEL,
            );

            let _ = device_without_signal.receive_signal(gps_signal, time);
            let _ = device_without_signal.update();
        }

        assert_eq!(
            device_without_signal.real_position_in_meters.x,
            original_position.x
        );
        assert_eq!(
            device_without_signal.real_position_in_meters.y,
            original_position.y
        );
        assert_eq!(
            device_without_signal.real_position_in_meters.z,
            original_position.z
        );
    }
    
    #[test]
    fn returning_to_home_on_signal_loss() {
        let home_point = Point3D::new(
            -MAX_DRONE_SPEED / 3.0, 
            -MAX_DRONE_SPEED / 3.0, 
            -MAX_DRONE_SPEED / 3.0
        );
        let signal_loss_response = SignalLossResponse::ReturnToHome(home_point);
        let destination_point = Point3D::new(
            MAX_DRONE_SPEED / 3.0, 
            MAX_DRONE_SPEED / 3.0, 
            MAX_DRONE_SPEED / 3.0
        );
        let task = Task::new(
            TaskType::Reposition,
            Some(destination_point)
        );
        
        let mut device_without_signal = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(device_power_system())
            .set_movement_system(drone_movement_system())
            .set_trx_system(drone_green_trx_system())
            .set_signal_loss_response(signal_loss_response)
            .build();

        let many_iterations = ITERATION_TIME * 500;
        for time in (0..many_iterations).step_by(ITERATION_TIME as usize) {
            let gps_data = Data::GPS(*device_without_signal.position());
            let gps_signal = Signal::new(
                SOME_DEVICE_ID,
                device_without_signal.id(),
                Some(gps_data), 
                GPS_L1_FREQUENCY,
                RED_SIGNAL_LEVEL,
            );
            
            assert!(
                device_without_signal.receive_signal(gps_signal, time).is_ok()
            );
            let _ = device_without_signal.update();
        }

        assert!(device_without_signal.at_destination(&home_point));
    }
    
    #[test]
    fn shutting_down_on_signal_loss() {
        let signal_loss_response = SignalLossResponse::Shutdown;
        let destination_point = Point3D::new(5.0, 5.0, 5.0);
        let task = Task::new(
            TaskType::Reposition,
            Some(destination_point)
        );
        
        let mut device_without_signal = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(device_power_system())
            .set_signal_loss_response(signal_loss_response)
            .build();

        let many_iterations = 500;
        for time in (0..many_iterations).step_by(ITERATION_TIME as usize) {
            let gps_data = Data::GPS(*device_without_signal.position());
            let gps_signal = Signal::new(
                SOME_DEVICE_ID,
                device_without_signal.id(),
                Some(gps_data), 
                GPS_L1_FREQUENCY,
                RED_SIGNAL_LEVEL,
            );

            let _ = device_without_signal.receive_signal(gps_signal, time);
            let _ = device_without_signal.update();
        }

        assert!(device_without_signal.is_shut_down());
    }
    
    #[test]
    fn no_movement_without_destination_set() {
        let device_position = Point3D::new(5.0, 0.0, 0.0);

        let mut device = DeviceBuilder::new()
            .set_real_position(device_position)
            .set_power_system(device_power_system())
            .set_movement_system(drone_movement_system())
            .set_trx_system(drone_green_trx_system())
            .build();

        assert_eq!(
            *device.gps_position(), 
            Point3D::default()
        );
        assert_eq!(
            *device.position(), 
            device_position
        );

        for _ in (0..1000).step_by(ITERATION_TIME as usize) {
            let _ = device.update();

            assert_eq!(
                *device.gps_position(), 
                Point3D::default()
            );
            assert_eq!(
                *device.position(), 
                device_position
            );
        }
    }

    #[test]
    fn device_movement_without_gps() {
        let destination_point = Point3D::new(MAX_DRONE_SPEED, 0.0, 0.0);
        let task = Task::new(
            TaskType::Reposition,
            Some(destination_point)
        );
        
        let mut device_without_gps = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(device_power_system())
            .set_movement_system(drone_movement_system())
            .build();

        for _ in (0..1000).step_by(ITERATION_TIME as usize) {
            let _ = device_without_gps.update();
        }

        assert_eq!(
            *device_without_gps.gps_position(), 
            Point3D::default()
        );
        assert_eq!(
            *device_without_gps.position(), 
            Point3D::default()
        );
    }

    #[test]
    fn device_reaching_destination() {
        let destination_point = Point3D::new(MAX_DRONE_SPEED, 0.0, 0.0);
        let task = Task::new(
            TaskType::Reposition,
            Some(destination_point)
        );
        let trx_system = TRXSystem::new( 
            TRXSystemType::Strength,
            TXModule::default(), 
            rx_module() 
        );
        
        let mut device = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(device_power_system())
            .set_movement_system(drone_movement_system())
            .set_trx_system(trx_system)
            .build();
            
        let many_iterations = 1000;
        for time in (0..many_iterations).step_by(ITERATION_TIME as usize) {
            let gps_data = Data::GPS(*device.position());
            let gps_signal = Signal::new(
                SOME_DEVICE_ID,
                device.id(),
                Some(gps_data), 
                GPS_L1_FREQUENCY,
                RED_SIGNAL_LEVEL,
            );
            
            assert!(device.receive_signal(gps_signal, time).is_ok()); 
            assert!(device.update().is_ok());
        }

        assert!(device.at_destination(&destination_point));
    }

    #[test]
    fn device_selfdestruction() {
        let task            = Task::new(
            TaskType::Attack,
            Some(Point3D::new(5.0, 5.0, 5.0))
        );
        let power_system    = device_power_system();
        let movement_system = drone_movement_system();
        let trx_system      = drone_green_trx_system();

        let mut device = DeviceBuilder::new()
            .set_task(task)
            .set_power_system(power_system.clone())
            .set_trx_system(trx_system.clone())
            .set_movement_system(movement_system.clone())
            .build();

        assert_eq!(device.task, task);
        assert_eq!(device.power_system, power_system);
        assert_eq!(device.trx_system, trx_system);
        assert_eq!(device.movement_system, movement_system);

        device.selfdestruction();

        assert!(device.is_shut_down());
    }

    #[test]
    fn receive_and_process_correct_set_task_signal() {
        let task = Task::new(
            TaskType::Attack,
            Some(Point3D::new(5.0, 0.0, 0.0))
        );

        let mut device = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(drone_green_trx_system())
            .build();
            
        let signal = Signal::new(
            SOME_DEVICE_ID,
            device.id(),
            Some(Data::SetTask(task)),
            CONTROL_FREQUENCY, 
            RED_SIGNAL_LEVEL, 
        );

        assert!(device.receive_signal(signal, 0).is_ok());
        assert!(device.process_received_signals().is_ok());
        assert_eq!(task, device.task);
    }
    
    #[test]
    fn receive_and_process_correct_gps_signal() {
        let global_position = Point3D::new(5.0, 0.0, 0.0);
        let gps_position    = Point3D::new(0.0, 0.0, 5.0);

        let mut device = DeviceBuilder::new()
            .set_real_position(global_position)
            .set_power_system(device_power_system())
            .set_trx_system(drone_green_trx_system())
            .build();
            
        assert_eq!(device.real_position_in_meters, global_position);
        assert_eq!(device.gps_position(), Point3D::default());

        let gps_signal = Signal::new(
            SOME_DEVICE_ID,
            device.id(),
            Some(Data::GPS(gps_position)), 
            GPS_L1_FREQUENCY,
            RED_SIGNAL_LEVEL,
        );

        assert!(device.receive_signal(gps_signal, 0).is_ok());
        assert!(device.process_received_signals().is_ok());
        assert_eq!(device.real_position_in_meters, global_position);
        assert_eq!(device.gps_position(), gps_position);
    }

    #[test]
    fn receive_and_process_broadcast_signal() {
        let task = Task::new(
            TaskType::Attack,
            Some(Point3D::new(5.0, 0.0, 0.0))
        );
        
        let mut device = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(drone_green_trx_system())
            .build();    
        
        let signal = Signal::new(
            SOME_DEVICE_ID,
            BROADCAST_ID,
            Some(Data::SetTask(task)), 
            CONTROL_FREQUENCY, 
            RED_SIGNAL_LEVEL, 
        );

        assert!(device.receive_signal(signal, 0).is_ok());
        assert!(device.process_received_signals().is_ok());
        assert_eq!(task, device.task);
    }

    #[test]
    fn not_receive_signal_with_wrong_destination() {
        let undefined_task = Task::default();
        let mut device = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(drone_green_trx_system())
            .build();
            
        let signal = Signal::new(
            SOME_DEVICE_ID,
            device.id() + 1,
            Some(Data::SetTask(undefined_task)), 
            CONTROL_FREQUENCY, 
            RED_SIGNAL_LEVEL, 
        );

        assert!(
            matches!(
                device.receive_signal(signal, 0),
                Err(TRXSystemError::WrongSignalDestination)
            )
        );
    }

    #[test]
    fn patched_device_does_not_get_infected() {
        let malware    = indicator_malware(); 
        let mut device = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(drone_green_trx_system())
            .build(); 
        
        let signal = Signal::new(
            SOME_DEVICE_ID,
            BROADCAST_ID,
            Some(Data::Malware(malware)), 
            CONTROL_FREQUENCY, 
            RED_SIGNAL_LEVEL, 
        );

        assert!(!device.is_infected());
        assert!(!device.is_infected_with(&malware));
        assert!(
            matches!(
                device.infection_state_on(&malware),
                InfectionState::Patched
            )
        );

        assert!(device.receive_signal(signal, 0).is_ok());
        
        assert!(!device.is_infected());
        assert!(!device.is_infected_with(&malware));
        assert!(
            matches!(
                device.infection_state_on(&malware),
                InfectionState::Patched
            )
        );
    }

    #[test]
    fn vulnerable_device_gets_infected() {
        let malware    = indicator_malware(); 
        let mut device = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(drone_green_trx_system())
            .set_vulnerabilities(&[malware])
            .build(); 
        
        let signal = Signal::new(
            SOME_DEVICE_ID,
            BROADCAST_ID,
            Some(Data::Malware(malware)), 
            CONTROL_FREQUENCY,
            RED_SIGNAL_LEVEL, 
        );

        assert!(!device.is_infected());
        assert!(!device.is_infected_with(&malware));
        assert!(
            matches!(
                device.infection_state_on(&malware),
                InfectionState::Vulnerable
            )
        );

        assert!(device.receive_signal(signal, 0).is_ok());
        assert!(device.process_received_signals().is_ok());

        assert!(device.is_infected());
        assert!(device.is_infected_with(&malware));
        assert!(
            matches!(
                device.infection_state_on(&malware),
                InfectionState::Infected
            )
        );
    }
}
