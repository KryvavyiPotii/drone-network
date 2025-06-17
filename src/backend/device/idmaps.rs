use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut, Keys, Values, ValuesMut};

use serde::Serialize;

use crate::backend::mathphysics::Millisecond;
use crate::backend::signal::SignalLevel;
use crate::backend::task::Task;

use super::{Device, DeviceId};


pub type IdToDelayMap = HashMap<DeviceId, Millisecond>;
pub type IdToLevelMap = HashMap<DeviceId, SignalLevel>;
pub type IdToTaskMap  = HashMap<DeviceId, Task>;


#[derive(Clone, Debug, Default, Serialize)]
pub struct IdToDeviceMap(HashMap<DeviceId, Device>);

impl IdToDeviceMap {
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub fn get(&self, device_id: &DeviceId) -> Option<&Device> {
        self.0.get(device_id)
    }
    
    #[must_use]
    pub fn get_mut(&mut self, device_id: &DeviceId) -> Option<&mut Device> {
        self.0.get_mut(device_id)
    }

    #[must_use]
    pub fn ids(&self) -> Keys<'_, DeviceId, Device> {
        self.0.keys()
    }

    #[must_use]
    pub fn devices(&self) -> Values<'_, DeviceId, Device> {
        self.0.values()
    }
    
    #[must_use]
    pub fn devices_mut(&mut self) -> ValuesMut<'_, DeviceId, Device> {
        self.0.values_mut()
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, DeviceId, Device> {
        self.0.iter()
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_, DeviceId, Device> {
        self.0.iter_mut()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    #[must_use]
    pub fn contains_id(&self, device_id: &DeviceId) -> bool {
        self.0.contains_key(device_id)
    }

    #[must_use]
    pub fn tasks(&self) -> IdToTaskMap {
        self.0
            .iter()
            .map(|(device_id, device)| (*device_id, *device.task()))
            .collect()
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&DeviceId, &mut Device) -> bool
    {
        self.0.retain(f);
    }
}

impl<'a> IntoIterator for &'a IdToDeviceMap{
    type Item = (&'a DeviceId, &'a Device);
    type IntoIter = Iter<'a, DeviceId, Device>;
    
    fn into_iter(self) -> Self::IntoIter {
         self.iter()
    }
}

impl<'a> IntoIterator for &'a mut IdToDeviceMap{
    type Item = (&'a DeviceId, &'a mut Device);
    type IntoIter = IterMut<'a, DeviceId, Device>;
    
    fn into_iter(self) -> Self::IntoIter {
         self.iter_mut()
    }
}

impl From<&[Device]> for IdToDeviceMap {
    fn from(devices: &[Device]) -> Self {
        let hash_map = devices 
            .iter()
            .map(|device| (device.id(), device.clone()))
            .collect();

        Self(hash_map)
    }
}

impl<const N: usize> From<[Device; N]> for IdToDeviceMap {
    fn from(devices: [Device; N]) -> Self {
        let hash_map = devices 
            .iter()
            .map(|device| (device.id(), device.clone()))
            .collect();
        
        Self(hash_map)
    }
}
