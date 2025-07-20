use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{CONTROL_FREQUENCY, ITERATION_TIME};
use super::connections::{ConnectionGraph, Topology};
use super::device::{Device, DeviceId, IdToDeviceMap};
use super::malware::Malware;
use super::mathphysics::Millisecond;
use super::signal::{Data, SignalQueue};
use super::task::Scenario;

use attack::{add_malware_signals_to_queue, AttackerDevice};
use gps::GPS;


pub mod attack;
pub mod gps;


#[derive(Clone, Default)]
pub struct NetworkModelBuilder {
    command_center_id: Option<DeviceId>,
    device_map: Option<IdToDeviceMap>,
    attacker_devices: Option<Vec<AttackerDevice>>,
    gps: Option<GPS>,
    topology: Option<Topology>,
    scenario: Option<Scenario>,
    delay_multiplier: Option<f32>,
}

impl NetworkModelBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_center_id: None,
            device_map: None,
            attacker_devices: None,
            gps: None,
            topology: None,
            scenario: None,
            delay_multiplier: None,
        }
    }

    #[must_use]
    pub fn set_command_center_id(
        mut self, 
        command_center_id: DeviceId
    ) -> Self {
        self.command_center_id = Some(command_center_id);
        self
    }

    #[must_use]
    pub fn set_device_map(mut self, device_map: IdToDeviceMap) -> Self {
        self.device_map = Some(device_map);
        self
    }

    #[must_use]
    pub fn set_attacker_devices(
        mut self, 
        attacker_devices: Vec<AttackerDevice> 
    ) -> Self {
        self.attacker_devices = Some(attacker_devices);
        self
    }
    
    #[must_use]
    pub fn set_gps(mut self, gps: GPS) -> Self {
        self.gps = Some(gps);
        self
    }

    #[must_use]
    pub fn set_topology(mut self, topology: Topology) -> Self {
        self.topology = Some(topology);
        self
    }

    #[must_use]
    pub fn set_scenario(mut self, scenario: Scenario) -> Self {
        self.scenario = Some(scenario);
        self
    }
    
    #[must_use]
    pub fn set_delay_multiplier(mut self, delay_multiplier: f32) -> Self {
        self.delay_multiplier = Some(delay_multiplier);
        self
    }

    #[must_use]
    pub fn build(self) -> NetworkModel {
        NetworkModel::new(
            self.command_center_id.unwrap_or_default(),
            self.device_map.unwrap_or_default(),
            self.attacker_devices.unwrap_or_default(),
            self.gps.unwrap_or_default(),
            self.scenario.unwrap_or_default(),
            self.topology.unwrap_or_default(),
            self.delay_multiplier.unwrap_or_default(),
        )
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkModel {
    current_time: Millisecond,
    command_device_id: DeviceId,
    device_map: IdToDeviceMap,
    attacker_devices: Vec<AttackerDevice>,
    gps: GPS,
    connections: ConnectionGraph,
    delay_multiplier: f32,
    scenario: Scenario,
    signal_queue: SignalQueue,
}

impl NetworkModel {
    #[must_use]
    pub fn new(
        command_device_id: DeviceId,
        device_map: IdToDeviceMap,
        attacker_devices: Vec<AttackerDevice>,
        gps: GPS,
        scenario: Scenario,
        topology: Topology,
        delay_multiplier: f32
    ) -> Self {
        let mut network_model = Self {
            current_time: 0,
            command_device_id,
            attacker_devices,
            device_map,
            gps,
            connections: ConnectionGraph::new(topology),
            delay_multiplier,
            scenario,
            signal_queue: SignalQueue::new(),
        };

        network_model.set_initial_state();

        network_model
    }
    
    #[must_use]
    pub fn command_device(&self) -> Option<&Device> {
        self.device_map.get(&self.command_device_id)
    }

    #[must_use]
    pub fn device_map(&self) -> &IdToDeviceMap {
        &self.device_map
    }

    #[must_use]
    pub fn attacker_devices(&self) -> &[AttackerDevice] {
        self.attacker_devices.as_slice()
    }   

    #[must_use]
    pub fn connections(&self) -> &ConnectionGraph {
        &self.connections
    }

    #[must_use]
    pub fn signal_queue(&self) -> &SignalQueue {
        &self.signal_queue
    }

    /// # Errors
    ///
    /// Will return `Err` if serialization fails.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
    
    /// # Errors
    ///
    /// Will return `Err` if serialization fails.
    /// 
    /// # Panics
    ///
    /// Will panic if it fails to read the file at `model_path`.
    pub fn from_json(model_path: &Path) -> serde_json::Result<Self> {
        let json_string = fs::read_to_string(model_path)
            .expect("Failed to read `.json` file");

        serde_json::from_str(&json_string)
    }

    pub fn update(&mut self) {
        self.spread_malware();
        self.update_devices();
        self.remove_shut_down_devices();   
        self.update_connections_graph();
        self.signal_queue.remove_old_signals(self.current_time);
     
        self.current_time += ITERATION_TIME;
        
        self.add_scenario_signals_to_queue();
        self.add_gps_signals_to_queue();
    }

    fn spread_malware(&mut self) {
        for (device_id, device) in &self.device_map {
            let malware_list: Vec<Malware> = device.malware_infections()
                .iter()
                .map(|(_, malware)| *malware)
                .collect();

            if malware_list.is_empty() {
                continue;
            }

            for (neighbor_id, neighbor_device) in &self.device_map {
                if neighbor_id == device_id {
                    continue;
                }

                add_malware_signals_to_queue(
                    device, 
                    neighbor_device, 
                    &malware_list, 
                    &mut self.signal_queue, 
                    self.current_time, 
                    self.delay_multiplier
                );
            }
        }
    }

    fn update_devices(&mut self) {
        self.attacker_devices
            .iter_mut()
            .for_each(|attacker_device| { 
                let _ = attacker_device.device_mut().update(); 
            });

        let _ = self.gps.device_mut().update();
        
        for (device_id, device) in &mut self.device_map {
            for attacker_device in &self.attacker_devices {
                let _ = attacker_device.execute_attack(
                    device, 
                    &mut self.signal_queue,
                    self.current_time,
                    self.delay_multiplier
                );
            }

            for signal in self.signal_queue.get_current_signals_for(
                *device_id,
                self.current_time
            ) {
                let _ = device.receive_signal(*signal, self.current_time);
            }

            let _ = device.update();
        }
    }

    fn remove_shut_down_devices(&mut self) {
        self.device_map.retain(|_, device| !device.is_shut_down());
    }
    
    fn update_connections_graph(&mut self) {
        self.connections.update(
            self.command_device_id, 
            &self.device_map,
            CONTROL_FREQUENCY,
        );
    }

    fn add_scenario_signals_to_queue(&mut self) {
        let Some(command_device) = self.device_map.get(
            &self.command_device_id
        ) else {
            return;
        };

        for (device_id, device) in &self.device_map {
            if *device_id == command_device.id() {
                continue;
            }

            let Some(last_task) = self.scenario.get_last_task(
                self.current_time, 
                *device_id
            ) else {
                continue;
            };

            let Ok(task_signal) = command_device.create_signal_for(
                device, 
                Some(Data::SetTask(*last_task)), 
                CONTROL_FREQUENCY
            ) else {
                continue;
            };
        
            let delay_map = self.connections.delay_map(
                command_device,
                *device_id, 
                &self.device_map, 
                self.delay_multiplier
            );

            self.signal_queue.add_entry(
                self.current_time, 
                task_signal, 
                delay_map
            );
        }
    }
   
    fn add_gps_signals_to_queue(&mut self) {
        self.gps.add_gps_signals_to_queue(
            &mut self.signal_queue, 
            &self.device_map, 
            self.current_time,
            self.delay_multiplier,
        );
    }

    fn set_initial_state(&mut self) {
        self.update_connections_graph();
        self.add_gps_signals_to_queue();
        self.add_scenario_signals_to_queue();
    }
}


#[cfg(test)]
mod tests {
    use attack::AttackType;

    use crate::backend::device::{DeviceBuilder, SignalLossResponse};
    use crate::backend::device::systems::{
        PowerSystem, RXModule, TXModule, TRXSystem, TRXSystemType
    };
    use crate::backend::mathphysics::{Megahertz, Meter, Point3D, PowerUnit};
    use crate::backend::malware::{Malware, MalwareType};
    use crate::backend::signal::{
        GREEN_SIGNAL_LEVEL, GPS_L1_FREQUENCY, FreqToLevelMap, SignalArea, 
        SignalLevel, 
    };
    
    use super::*;


    const DRONE_TX_CONTROL_RADIUS: Meter = 10.0;
    const DEFAULT_GPS_POSITION_IN_METERS: Point3D = Point3D { 
        x: 0.0, 
        y: 0.0, 
        z: GPS_TX_RADIUS / 2.0
    };
    const DEVICE_MAX_POWER: PowerUnit   = 10_000;
    const GPS_TX_RADIUS: Meter          = 1_000.0;

    
    fn tx_module(frequency: Megahertz, radius: Meter) -> TXModule {
        let tx_signal_level  = SignalLevel::from_area(
            SignalArea::build(radius).unwrap(), 
            frequency
        );
        let tx_signal_levels = FreqToLevelMap::from([
            (frequency, tx_signal_level)
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

    fn default_gps() -> GPS {
        let trx_system = TRXSystem::new( 
            TRXSystemType::Strength,
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

    fn device_power_system() -> PowerSystem {
        PowerSystem::build(DEVICE_MAX_POWER, DEVICE_MAX_POWER)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn indicator_malware() -> Malware {
        Malware::new(MalwareType::Indicator, 0, Some(0))
    }

    fn malware_distributor(
        position: Point3D, 
        malware: Malware
    ) -> AttackerDevice {
        let trx_system = TRXSystem::new( 
            TRXSystemType::Strength,
            tx_module(CONTROL_FREQUENCY, DRONE_TX_CONTROL_RADIUS),
            RXModule::default()
        );

        let device = DeviceBuilder::new()
            .set_real_position(position)
            .set_signal_loss_response(SignalLossResponse::Ignore)
            .set_power_system(device_power_system())
            .set_trx_system(trx_system)
            .build();

        AttackerDevice::new(device, AttackType::MalwareDistribution(malware))
    }


    #[test]
    fn spread_malware_in_star() {
        let indicator_malware = indicator_malware();
        let vulnerable_device_builder = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_vulnerabilities(&[indicator_malware])
            .set_trx_system(
                TRXSystem::new( 
                    TRXSystemType::Strength,
                    tx_module(CONTROL_FREQUENCY, DRONE_TX_CONTROL_RADIUS),
                    rx_module() 
                )
            );
        
        // Network topology:
        //
        // B -(5.0)- A -(5.0)- C -(5.0)- M
        //
        let command_center = vulnerable_device_builder
            .clone()
            .build();
        let command_center_id = command_center.id();
        
        let infected_drone = vulnerable_device_builder
            .clone()
            .set_real_position(Point3D::new(5.0, 0.0, 0.0))
            .build();
        let infected_drone_id = infected_drone.id();

        let vulnerable_drone = vulnerable_device_builder
            .set_real_position(Point3D::new(-5.0, 0.0, 0.0))
            .build();
        let vulnerable_drone_id = vulnerable_drone.id();

        let device_map = IdToDeviceMap::from(
            [command_center, infected_drone, vulnerable_drone]
        );
        
        let malware_distributor = malware_distributor(
            Point3D::new(10.0, 0.0, 0.0),
            indicator_malware
        );
       
        let mut network_model = NetworkModel::new(
            command_center_id, 
            device_map, 
            vec![malware_distributor], 
            default_gps(),
            Scenario::new(), 
            Topology::Star, 
            0.0
        );

        assert!(
            !network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );

        network_model.update();
        
        assert!(
            network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );

        network_model.update();

        assert!(
            network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );

        network_model.update();

        assert!(
            network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            network_model.device_map
                .get(&vulnerable_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );
    }

    #[test]
    fn spread_malware_in_mesh() {
        let indicator_malware = indicator_malware();
        let vulnerable_device_builder = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_vulnerabilities(&[indicator_malware])
            .set_trx_system(
                TRXSystem::new( 
                    TRXSystemType::Strength,
                    tx_module(CONTROL_FREQUENCY, DRONE_TX_CONTROL_RADIUS),
                    rx_module() 
                )
            );
        
        let command_center = vulnerable_device_builder
            .clone()
            .build();
        let command_center_id = command_center.id();
        
        let infected_drone = vulnerable_device_builder
            .clone()
            .set_real_position(Point3D::new(10.0, 0.0, 0.0))
            .build();
        let infected_drone_id = infected_drone.id();

        let vulnerable_drone1 = vulnerable_device_builder
            .clone()
            .set_real_position(Point3D::new(1.1, 0.0, 0.0))
            .build();
        let vulnerable_drone_id1 = vulnerable_drone1.id();

        let vulnerable_drone2 = vulnerable_device_builder
            .clone()
            .set_real_position(Point3D::new(2.0, 0.0, 0.0))
            .build();
        let vulnerable_drone_id2 = vulnerable_drone2.id();

        let vulnerable_drone3 = vulnerable_device_builder
            .clone()
            .set_real_position(Point3D::new(3.0, 0.0, 0.0))
            .build();
        let vulnerable_drone_id3 = vulnerable_drone3.id();
        
        let devices = [
            command_center,
            infected_drone, 
            vulnerable_drone1,
            vulnerable_drone2,
            vulnerable_drone3,
        ];

        let malware_distributor = malware_distributor(
            Point3D::new(15.0, 0.0, 0.0),
            indicator_malware
        );
        
        let mut network_model = NetworkModel::new(
            command_center_id, 
            IdToDeviceMap::from(devices), 
            vec![malware_distributor], 
            default_gps(),
            Scenario::new(), 
            Topology::Mesh, 
            0.0
        );

        assert!(
            !network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id1)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id2)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id3)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );

        network_model.update();       
        
        assert!(
            network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id1)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id2)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model.device_map
                .get(&vulnerable_drone_id3)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );

        network_model.update();

        assert!(
            network_model.device_map
                .get(&infected_drone_id)
                .unwrap()
                .is_infected()
        );
        assert!(
            network_model.device_map
                .get(&vulnerable_drone_id1)
                .unwrap()
                .is_infected()
        );
        assert!(
            network_model.device_map
                .get(&vulnerable_drone_id2)
                .unwrap()
                .is_infected()
        );
        assert!(
            network_model.device_map
                .get(&vulnerable_drone_id3)
                .unwrap()
                .is_infected()
        );
        assert!(
            !network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );
        
        network_model.update();
        
        assert!(
            network_model
                .command_device()
                .expect("Missing command device")
                .is_infected()
        );
    }
}
