use std::collections::HashMap;
use std::fmt;

use rustworkx_core::distancemap::DistanceMap;
use serde::{self, Serialize};
use serde::ser::{Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use thiserror::Error;

use petgraph::Directed;
use petgraph::graphmap::GraphMap; 
use petgraph::visit::EdgeRef;
use rustworkx_core::centrality::betweenness_centrality;
use rustworkx_core::dictmap::DictMap;
use rustworkx_core::shortest_path::{astar, dijkstra};

use super::device::{
    Device, DeviceId, IdToDelayMap, IdToDeviceMap, BROADCAST_ID
};
use super::mathphysics::{delay_to, Megahertz, Meter, Position};
use super::signal::SignalLevel;


type Connection<'a> = (DeviceId, DeviceId, &'a (Meter, SignalLevel));
type SerdeEdge      = (DeviceId, DeviceId, (Meter, SignalLevel));
type ConnectionMap  = GraphMap<DeviceId, (Meter, SignalLevel), Directed>;


#[derive(Error, Debug)]
pub enum ShortestPathError {
    #[error("Shortest path was not found")]
    NoPathFound,
    #[error("Path length is less than 2")]
    PathTooShort
}
    

fn unicast_delay_map_from_outside_network(
    destination_id: DeviceId,
    source_device: &Device,
    device_map: &IdToDeviceMap,
    delay_multiplier: f32,
) -> IdToDelayMap {
    let Some(destination_device) = device_map.get(&destination_id) else {
        return IdToDelayMap::new();
    };
    
    let delay = delay_to(
        source_device.distance_to(destination_device), 
        delay_multiplier
    );

    IdToDelayMap::from([(destination_id, delay)])
}


#[derive(Clone, Copy, Debug, Default, Serialize, serde::Deserialize)]
pub enum Topology {
    #[default]
    Star,
    Mesh,
}


#[derive(Clone, Debug, Default)]
pub struct ConnectionGraph {
    graph_map: ConnectionMap, 
    topology: Topology,
}

impl ConnectionGraph {
    #[must_use]
    pub fn new(topology: Topology) -> Self {
        Self { 
            graph_map: GraphMap::new(),
            topology
        }
    }

    #[must_use]
    pub fn graph_map(&self) -> &ConnectionMap {
        &self.graph_map
    }

    // Currently, it considers only distances between devices while building the 
    // most efficient paths. It ignores signal levels of devices.
    pub fn update(
        &mut self, 
        command_device_id: DeviceId,
        device_map: &IdToDeviceMap,
        frequency: Megahertz
    ) {
        self.graph_map.clear();
        
        let Some(command_device) = device_map.get(&command_device_id) else {
            return 
        };

        match self.topology {
            Topology::Star => 
                self.create_star(command_device, device_map, frequency),
            Topology::Mesh => 
                self.create_mesh(device_map, frequency),
        }
    }

    fn create_star(
        &mut self,
        central_device: &Device,
        device_map: &IdToDeviceMap,
        frequency: Megahertz
    ) {
        for device in device_map.devices() {
            self.connect_devices(central_device, device, frequency);    
        }
    }

    fn create_mesh(
        &mut self, 
        device_map: &IdToDeviceMap,
        frequency: Megahertz
    ) {
        for tx in device_map.devices() {
            for rx in device_map.devices() {
                self.connect_devices(tx, rx, frequency);    
            }
        }
    }

    fn connect_devices(
        &mut self,
        device1: &Device,
        device2: &Device,
        frequency: Megahertz,
    ) {
        // Loops are prohibited. Otherwise, shortest path algorithms will 
        // not function properly.
        if device1.id() == device2.id() {
            return;
        }

        let distance = device2.distance_to(device1);

        if let Some(tx_signal_level_from_device1) = device1.tx_signal_level_at(
            device2, 
            frequency
        ) {
            self.graph_map.add_edge(
                device1.id(), 
                device2.id(), 
                (distance, tx_signal_level_from_device1)
            );
        }
        if let Some(tx_signal_level_from_device2) = device2.tx_signal_level_at(
            device1, 
            frequency
        ) {   
            self.graph_map.add_edge(
                device2.id(), 
                device1.id(), 
                (distance, tx_signal_level_from_device2)
            );
        }
    }
    
    #[must_use]
    pub fn delay_map(
        &self,
        source_device: &Device,
        destination_id: DeviceId,
        device_map: &IdToDeviceMap,
        delay_multiplier: f32,
    ) -> IdToDelayMap {
        if self.graph_map.contains_node(source_device.id()) {
            self.delay_map_from_inside_network(
                source_device.id(), 
                destination_id,
                delay_multiplier
            )
        } else {
            self.delay_map_from_outside_network(
                source_device, 
                destination_id,
                device_map,
                delay_multiplier
            )        
        }
    }

    fn delay_map_from_inside_network(
        &self,
        source: DeviceId,
        destination: DeviceId,
        delay_multiplier: f32,
    ) -> IdToDelayMap {
        if destination == BROADCAST_ID {
            return self.broadcast_delay_map_from_inside_network(
                source, 
                delay_multiplier
            );
        }

        self.unicast_delay_map_from_inside_network(
            source, 
            destination, 
            delay_multiplier
        )
    }
    
    fn unicast_delay_map_from_inside_network(
        &self, 
        source: DeviceId,
        destination: DeviceId,
        delay_multiplier: f32
    ) -> IdToDelayMap {
        let distance_map = self
            .dijkstra(source, Some(destination))
            .unwrap();
        
        let distance = distance_map
            .get_item(destination)
            .unwrap();

        IdToDelayMap::from([
            (destination, delay_to(*distance, delay_multiplier))
        ])
    }
 
    fn broadcast_delay_map_from_inside_network(
        &self, 
        source: DeviceId,
        delay_multiplier: f32
    ) -> IdToDelayMap {
        let distance_map = self.dijkstra(source, None).unwrap();

        distance_map
            .iter()
            .map(|(device_id, distance)| {
                let delay = delay_to(*distance, delay_multiplier);
            
                (*device_id, delay)
            })
            .collect()
    }

    fn delay_map_from_outside_network(
        &self,
        source_device: &Device,
        destination_id: DeviceId,
        device_map: &IdToDeviceMap,
        delay_multiplier: f32,
    ) -> IdToDelayMap {
        if destination_id == BROADCAST_ID {
            return self.broadcast_delay_map_from_outside_network(
                destination_id, 
                source_device, 
                device_map, 
                delay_multiplier
            );
        } 

        unicast_delay_map_from_outside_network(
            destination_id, 
            source_device, 
            device_map, 
            delay_multiplier
        )
    }

    fn broadcast_delay_map_from_outside_network(
        &self,
        destination_id: DeviceId,
        source_device: &Device,
        device_map: &IdToDeviceMap,
        delay_multiplier: f32,
    ) -> IdToDelayMap {
        self.graph_map
            .nodes()
            .filter_map(|device_id| {
                let destination_device = device_map.get(&destination_id)?; 
                
                let delay = delay_to(
                    source_device.distance_to(destination_device), 
                    delay_multiplier
                );

                Some((device_id, delay))
            })
            .collect()
    }

    // Gives shortest distance to a device by distance between devices.
    /// # Errors
    ///
    /// Will never fail.
    pub fn dijkstra(
        &self,
        source: DeviceId,
        destination: Option<DeviceId>,
    ) -> rustworkx_core::Result<DictMap<DeviceId, f32>> {
        dijkstra(
            &self.graph_map,
            source,
            destination,
            |edge| Ok(edge.weight().0),
            None
        )
    }

    // Gives distance and path to a device by distance between devices.
    /// # Errors
    ///
    /// Will return `Err` if the shortest path algorithm does not find an 
    /// appropriate path.
    pub fn find_shortest_path_from_to(
        &self,
        source: DeviceId,
        destination: DeviceId 
    ) -> Result<(Meter, Vec<DeviceId>), ShortestPathError> {
        let Ok(Some((distance, path))) = astar(
            &self.graph_map,
            source,
            |finish| -> rustworkx_core::Result<bool> {
                Ok(finish == destination)
            },
            |edge| Ok(edge.weight().0),
            |_| Ok(0.0)
        ) else {
            return Err(ShortestPathError::NoPathFound);
        };

        if path.len() < 2 {
            return Err(ShortestPathError::PathTooShort);
        } 
        
        Ok((distance, path))
    }

    #[must_use]
    pub fn all_incoming_degrees(&self) -> HashMap<DeviceId, usize> {
        self.graph_map
            .nodes()
            .map(|node| {
                let incoming_degree = self.graph_map
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .count();

                (node, incoming_degree)
            })
            .collect()
    }

    #[must_use]
    pub fn all_outgoing_degrees(&self) -> HashMap<DeviceId, usize> {
        self.graph_map
            .nodes()
            .map(|node| {
                let outgoing_degree = self.graph_map
                    .neighbors_directed(node, petgraph::Direction::Outgoing)
                    .count();

                (node, outgoing_degree)
            })
            .collect()
    }
    
    /// # Panics
    /// 
    /// Will panic if `rustworkx_core::shortest_path::dijkstra` becomes 
    /// fallible.
    #[must_use]
    pub fn diameter(&self) -> f32 {
        let shortest_paths: Vec<DictMap<DeviceId, f32>> = self.graph_map
            .nodes()
            .map(|drone_id| self.dijkstra(drone_id, None).unwrap())
            .collect();

        shortest_paths
            .iter()
            .flat_map(|dictmap| dictmap.values())
            .fold(0f32, |a, &b| a.max(b))
    }

    #[must_use]
    pub fn betweenness_centrality(&self) -> Vec<Option<f64>> {
        betweenness_centrality(&self.graph_map, true, true, 50)
    }
}

impl Serialize for ConnectionGraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        let mut state = serializer.serialize_struct("ConnectionGraph", 2)?;

        let all_edges: Vec<Connection> = self.graph_map
            .all_edges()
            .collect();

        state.serialize_field("edges", &all_edges)?;
        state.serialize_field("topology", &self.topology)?;
        state.end()    
    }
}

impl<'de> Deserialize<'de> for ConnectionGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Edges, Topology }
        struct ConnectionGraphVisitor;

        impl<'de> Visitor<'de> for ConnectionGraphVisitor {
            type Value = ConnectionGraph;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ConnectionGraph")
            }

            fn visit_seq<V>(
                self, 
                mut seq: V
            ) -> Result<ConnectionGraph, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let edges: Vec<SerdeEdge> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let graph_map = GraphMap::from_edges(edges);
                
                let topology = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(ConnectionGraph { graph_map, topology } )
            }

            fn visit_map<V>(
                self, 
                mut map: V
            ) -> Result<ConnectionGraph, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut edges = None;
                let mut topology = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Edges => {
                            if edges.is_some() {
                                return Err(
                                    de::Error::duplicate_field("edges")
                                );
                            }
                            edges = Some(map.next_value()?);
                        }
                        Field::Topology => {
                            if topology.is_some() {
                                return Err(
                                    de::Error::duplicate_field("topology")
                                );
                            }
                            topology = Some(map.next_value()?);
                        }
                    }
                }
                let edges: Vec<SerdeEdge> = edges
                    .ok_or_else(|| de::Error::missing_field("edges"))?;
                let graph_map = GraphMap::from_edges(edges);
                
                let topology = topology
                    .ok_or_else(|| de::Error::missing_field("topology"))?;

                Ok(ConnectionGraph { graph_map, topology } )
            }
        }

        const FIELDS: &[&str] = &["edges", "topology"];
        deserializer.deserialize_struct(
            "ConnectionGraph", 
            FIELDS, 
            ConnectionGraphVisitor
        )
    }
}


#[cfg(test)]
mod tests {
    use crate::backend::CONTROL_FREQUENCY;
    use crate::backend::device::{Device, DeviceBuilder};
    use crate::backend::device::systems::{
        PowerSystem, RXModule, TRXSystem, TRXSystemType, TXModule
    };
    use crate::backend::mathphysics::{Point3D, PowerUnit};
    use crate::backend::signal::{
        FreqToLevelMap, GREEN_SIGNAL_LEVEL, SignalLevel, SignalArea, 
        GPS_L1_FREQUENCY
    };
    
    use super::*;
    

    const CC_TX_CONTROL_RADIUS: Meter    = 300.0;
    const DEVICE_MAX_POWER: PowerUnit    = 1_000;
    const DRONE_TX_CONTROL_RADIUS: Meter = 10.0;
    

    fn device_power_system() -> PowerSystem {
        PowerSystem::build(DEVICE_MAX_POWER, DEVICE_MAX_POWER)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn round_with_precision(value: f32, precision: u8) -> f32 {
        let coefficient = 10.0_f32.powi(precision.into());

        (value * coefficient).round() / coefficient
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

    fn drone_with_trx_system_set(position: Point3D) -> Device {
        let trx_system = TRXSystem::new(
            TRXSystemType::Strength,
            control_tx_module(DRONE_TX_CONTROL_RADIUS),
            rx_module()
        );
        
        DeviceBuilder::new()
            .set_real_position(position)
            .set_power_system(device_power_system())
            .set_trx_system(trx_system)
            .build()
    }

    fn simple_mesh() -> (ConnectionGraph, Vec<DeviceId>) {
        let frequency = CONTROL_FREQUENCY;
        
        // Network:
        //                      D
        //                      |
        //                    (7.28)
        //                      |
        //  A -(7.0)- B -(9.0)- C
        //                      |
        //                    (7.28)
        //                      |
        //                      E
        //
        let command_center = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(
                TRXSystem::new( 
                    TRXSystemType::Strength,
                    control_tx_module(DRONE_TX_CONTROL_RADIUS),
                    rx_module()
                )
            )
            .build();
        let command_center_id = command_center.id();
        
        let devices = [
            command_center,
            drone_with_trx_system_set(Point3D::new(7.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(14.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(16.0, 7.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(16.0, -7.0, 0.0)),
        ];
        let device_ids: Vec<DeviceId> = devices
            .iter()
            .map(|device| device.id())
            .collect();
    
        let device_map = IdToDeviceMap::from(devices);

        let mut connections = ConnectionGraph::new(Topology::Mesh);
        connections.update(
            command_center_id, 
            &device_map, 
            frequency
        );

        (connections, device_ids)
    }

    fn simple_star() -> (ConnectionGraph, Vec<DeviceId>) {
        let frequency = CONTROL_FREQUENCY;
        
        let command_center = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(
                TRXSystem::new( 
                    TRXSystemType::Strength,
                    control_tx_module(CC_TX_CONTROL_RADIUS),
                    rx_module()
                )
            )
            .build();
        let command_center_id = command_center.id();

        let devices = [
            command_center,
            drone_with_trx_system_set(Point3D::new(25.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(0.0, 25.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(0.0, 0.0, 25.0)),
        ];
        let device_ids: Vec<DeviceId> = devices
            .iter()
            .map(|device| device.id())
            .collect();

        let device_map = IdToDeviceMap::from(devices);

        let mut connections = ConnectionGraph::new(Topology::Star);
        connections.update(
            command_center_id, 
            &device_map, 
            frequency
        );

        (connections, device_ids)
    }


    #[test]
    fn create_star_connection_graph() {
        let (connections, device_ids) = simple_star(); 
        
        let cc_id = device_ids[0];
        let drone_b_id = device_ids[1];
        let drone_c_id = device_ids[2];
        let drone_d_id = device_ids[3];

        assert_eq!(3, connections.graph_map.edge_count());

        assert!(connections.graph_map.contains_edge(cc_id, drone_b_id));
        assert!(connections.graph_map.contains_edge(cc_id, drone_c_id));
        assert!(connections.graph_map.contains_edge(cc_id, drone_d_id));
    }

    #[test]
    fn create_mesh_connection_graph() {
        let (connections, device_ids) = simple_mesh(); 

        let cc_id = device_ids[0];
        let drone_b_id = device_ids[1];
        let drone_c_id = device_ids[2];
        let drone_d_id = device_ids[3];
        let drone_e_id = device_ids[4];

        assert_eq!(8, connections.graph_map.edge_count());
        
        assert!(connections.graph_map.contains_edge(cc_id, drone_b_id));
        assert!(connections.graph_map.contains_edge(drone_b_id, cc_id));
        
        assert!(connections.graph_map.contains_edge(drone_b_id, drone_c_id));
        assert!(connections.graph_map.contains_edge(drone_c_id, drone_b_id));
        
        assert!(connections.graph_map.contains_edge(drone_c_id, drone_d_id));
        assert!(connections.graph_map.contains_edge(drone_d_id, drone_c_id));

        assert!(connections.graph_map.contains_edge(drone_c_id, drone_e_id));
        assert!(connections.graph_map.contains_edge(drone_e_id, drone_c_id));
    }

    #[test]
    fn all_degrees_in_mesh() {
        let (connections, device_ids) = simple_mesh(); 
        
        let cc_id = device_ids[0];
        let drone_b_id = device_ids[1];
        let drone_c_id = device_ids[2];
        let drone_d_id = device_ids[3];
        let drone_e_id = device_ids[4];
        
        let expected_incoming_degrees = HashMap::from([
            (cc_id, 1),
            (drone_b_id, 2),
            (drone_c_id, 3),
            (drone_d_id, 1),
            (drone_e_id, 1),
        ]);
        let expected_outgoing_degrees = HashMap::from([
            (cc_id, 1),
            (drone_b_id, 2),
            (drone_c_id, 3),
            (drone_d_id, 1),
            (drone_e_id, 1),
        ]);

        assert_eq!(
            expected_incoming_degrees, 
            connections.all_incoming_degrees()
        );
        assert_eq!(
            expected_outgoing_degrees, 
            connections.all_outgoing_degrees()
        );
    }
    
    #[test]
    fn all_degrees_in_star() {
        let (connections, device_ids) = simple_star(); 
        
        let cc_id = device_ids[0];
        let drone_b_id = device_ids[1];
        let drone_c_id = device_ids[2];
        let drone_d_id = device_ids[3];
        
        let expected_incoming_degrees = HashMap::from([
            (cc_id, 0),
            (drone_b_id, 1),
            (drone_c_id, 1),
            (drone_d_id, 1),
        ]);
        let expected_outgoing_degrees = HashMap::from([
            (cc_id, 3),
            (drone_b_id, 0),
            (drone_c_id, 0),
            (drone_d_id, 0),
        ]);

        assert_eq!(
            expected_incoming_degrees, 
            connections.all_incoming_degrees()
        );
        assert_eq!(
            expected_outgoing_degrees, 
            connections.all_outgoing_degrees()
        );
    }

    #[test]
    fn network_diameter() {
        let frequency = CONTROL_FREQUENCY;
        
        // Network 1: full mesh with edge weight 1.0.
        let command_center = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(
                TRXSystem::new( 
                    TRXSystemType::Strength,
                    control_tx_module(DRONE_TX_CONTROL_RADIUS),
                    RXModule::default() 
                )
            )
            .build();
        let command_center_id = command_center.id();
        
        let devices1 = IdToDeviceMap::from([
            command_center.clone(),
            drone_with_trx_system_set(Point3D::new(1.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(2.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(3.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(4.0, 0.0, 0.0)),
        ]);

        let mut connections = ConnectionGraph::new(Topology::Mesh);
        connections.update(
            command_center_id,
            &devices1,
            frequency
        );

        assert_eq!(connections.diameter(), 4.0);

        // Network 2:
        // 
        // A -(7.0)- B -(7.0)- C -(7.0)- D -(7.0)- E 
        //
        let devices2 = IdToDeviceMap::from([
            command_center.clone(),
            drone_with_trx_system_set(Point3D::new(7.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(14.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(21.0, 0.0, 0.0)),
            drone_with_trx_system_set(Point3D::new(28.0, 0.0, 0.0)),
        ]);
        
        connections.update(
            command_center_id,
            &devices2,
            frequency
        );

        assert_eq!(connections.diameter(), 28.0);
        
        // Network 3:
        //                      D
        //                      |
        //                    (7.28)
        //                      |
        //  A -(7.0)- B -(9.0)- C
        //                      |
        //                    (7.28)
        //                      |
        //                      E
        //
        let (connections, _) = simple_mesh();

        let diameter3 = round_with_precision(connections.diameter(), 2);

        assert_eq!(diameter3, 21.28);
    }
}
