use std::fmt;

use serde::{self, Serialize};
use serde::ser::{Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use thiserror::Error;

use petgraph::Directed;
use petgraph::graphmap::GraphMap; 
use petgraph::visit::EdgeRef;
use rustworkx_core::dictmap::DictMap;
use rustworkx_core::shortest_path::{astar, dijkstra};

use super::device::{
    Device, DeviceId, IdToDelayMap, IdToDeviceMap, BROADCAST_ID
};
use super::mathphysics::{delay_to, Frequency, Meter, Position};
use super::signal::SignalStrength;


type Connection<'a> = (DeviceId, DeviceId, &'a (Meter, SignalStrength));
type SerdeEdge      = (DeviceId, DeviceId, (Meter, SignalStrength));
type ConnectionMap  = GraphMap<DeviceId, (Meter, SignalStrength), Directed>;


#[derive(Error, Debug)]
pub enum ShortestPathError {
    #[error("Shortest path was not found")]
    NoPathFound,
    #[error("Path length is less than 2")]
    PathTooShort
}
    

#[derive(Clone, Copy, Debug, Default, Serialize, serde::Deserialize)]
pub enum Topology {
    Mesh,
    #[default]
    Star,
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
    // most efficient paths. It ignores signal qualities of devices.
    pub fn update(
        &mut self, 
        command_device_id: DeviceId,
        device_map: &IdToDeviceMap,
    ) {
        self.graph_map.clear();
        
        let Some(command_device) = device_map.get(&command_device_id) else {
            return 
        };

        match self.topology {
            Topology::Star => self.create_star(command_device, device_map),
            Topology::Mesh => self.create_mesh(device_map),
        }
    }

    fn create_star(
        &mut self,
        central_device: &Device,
        device_map: &IdToDeviceMap,
    ) {
        for device in device_map.values() {
            self.connect_devices(central_device, device); 
        }
    }

    fn create_mesh(&mut self, device_map: &IdToDeviceMap) {
        for tx in device_map.values() {
            for rx in device_map.values() {
                self.connect_devices(tx, rx);    
            }
        }
    }

    fn connect_devices(&mut self, device1: &Device, device2: &Device) {
        // Loops are prohibited. Otherwise, shortest path algorithms will 
        // not function properly.
        if device1.id() == device2.id() {
            return;
        }

        let distance = device2.distance_to(device1);

        self.connect_devices_in_one_direction(device1, device2, distance);
        self.connect_devices_in_one_direction(device2, device1, distance);
    }

    fn connect_devices_in_one_direction(
        &mut self,
        device1: &Device,
        device2: &Device,
        distance: Meter,
    ) {
        if let Some(tx_signal_strength_from_1) = device1.tx_signal_strength_at(
            device2, 
            Frequency::Control
        ) {
            if tx_signal_strength_from_1.is_black() {
                return;
            }

            self.graph_map.add_edge(
                device1.id(), 
                device2.id(), 
                (distance, tx_signal_strength_from_1)
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
        let distance_map = self.dijkstra(source, destination)
            .unwrap_or_else(|error| panic!("{}", error));

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
        let destination_ids: Vec<DeviceId> = if destination_id == BROADCAST_ID {
            self.graph_map.nodes().collect()
        } else if self.graph_map.contains_node(destination_id) {
            vec![destination_id]
        } else {
            Vec::new()
        };

        destination_ids
            .iter()
            .filter_map(|destination_id| {
                let destination_device = device_map.get(destination_id)?; 
                
                let delay = delay_to(
                    source_device.distance_to(destination_device), 
                    delay_multiplier
                );

                Some((*destination_id, delay))
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
        destination: DeviceId,
    ) -> rustworkx_core::Result<DictMap<DeviceId, f32>> {
        let destination = if destination == BROADCAST_ID {
            None
        } else {
            Some(destination)
        };

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
    use crate::backend::device::{Device, DeviceBuilder, device_map_from_slice};
    use crate::backend::device::systems::{
        PowerSystem, RXModule, TRXSystem, TXModule, 
    };
    use crate::backend::mathphysics::{Megahertz, Point3D, PowerUnit};
    use crate::backend::signal::{
        FreqToStrengthMap, GREEN_SIGNAL_STRENGTH, SignalStrength
    };
    
    use super::*;
    

    const CC_TX_CONTROL_RADIUS: Meter    = 300.0;
    const DEVICE_MAX_POWER: PowerUnit    = 1_000;
    const DRONE_TX_CONTROL_RADIUS: Meter = 10.0;
    

    fn device_power_system() -> PowerSystem {
        PowerSystem::build(DEVICE_MAX_POWER, DEVICE_MAX_POWER)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    fn control_trx_system(tx_area_radius: Meter) -> TRXSystem {
        TRXSystem::new(
            control_tx_module(tx_area_radius),
            rx_module(),
        )
    }

    fn control_tx_module(radius: Meter) -> TXModule {
        let tx_signal_strength  = SignalStrength::from_area_radius(
            radius,
            Frequency::Control as Megahertz
        );
        let tx_signal_qualities = FreqToStrengthMap::from([
            (Frequency::Control, tx_signal_strength)
        ]);

        TXModule::new(tx_signal_qualities)
    }
    
    fn rx_module() -> RXModule {
        let max_rx_signal_qualities = FreqToStrengthMap::from([
            (Frequency::Control, GREEN_SIGNAL_STRENGTH)
        ]);

        RXModule::new(max_rx_signal_qualities)
    }

    fn drone_with_trx_system_set(position: Point3D) -> Device {
        DeviceBuilder::new()
            .set_real_position(position)
            .set_power_system(device_power_system())
            .set_trx_system(control_trx_system(DRONE_TX_CONTROL_RADIUS))
            .build()
    }

    fn simple_mesh() -> (ConnectionGraph, Vec<DeviceId>) {
        // Network topology:
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
        let command_center = drone_with_trx_system_set(Point3D::default());
        let command_center_id = command_center.id();
        
        let devices = [
            command_center,                                           // A
            drone_with_trx_system_set(Point3D::new(7.0, 0.0, 0.0)),   // B
            drone_with_trx_system_set(Point3D::new(14.0, 0.0, 0.0)),  // C
            drone_with_trx_system_set(Point3D::new(16.0, 7.0, 0.0)),  // D
            drone_with_trx_system_set(Point3D::new(16.0, -7.0, 0.0)), // E
        ];
        let device_ids: Vec<DeviceId> = devices
            .iter()
            .map(|device| device.id())
            .collect();
        let device_map = device_map_from_slice(&devices);

        let mut connections = ConnectionGraph::new(Topology::Mesh);

        connections.update(command_center_id, &device_map);

        (connections, device_ids)
    }

    fn simple_star() -> (ConnectionGraph, Vec<DeviceId>) {
        // Network topology:
        //
        //                 C
        //                /
        //             (25.0)
        //              /
        //  B -(25.0)- A
        //             |
        //           (25.0)
        //             |
        //             D
        //
        let command_center = DeviceBuilder::new()
            .set_power_system(device_power_system())
            .set_trx_system(control_trx_system(CC_TX_CONTROL_RADIUS))
            .build();
        let command_center_id = command_center.id();

        let devices = [
            command_center,                                          // A
            drone_with_trx_system_set(Point3D::new(25.0, 0.0, 0.0)), // B
            drone_with_trx_system_set(Point3D::new(0.0, 25.0, 0.0)), // C
            drone_with_trx_system_set(Point3D::new(0.0, 0.0, 25.0)), // D
        ];
        let device_ids: Vec<DeviceId> = devices
            .iter()
            .map(|device| device.id())
            .collect();
        let device_map = device_map_from_slice(&devices);

        let mut connections = ConnectionGraph::new(Topology::Star);
        
        connections.update(command_center_id, &device_map);

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
}
