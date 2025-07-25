use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::backend::mathphysics::Millisecond;
use crate::backend::task::Task;

use super::Device;


pub type DeviceId = usize;
pub type IdToDelayMap  = HashMap<DeviceId, Millisecond>;
pub type IdToDeviceMap = HashMap<DeviceId, Device>;
pub type IdToTaskMap   = HashMap<DeviceId, Task>;


pub const BROADCAST_ID: DeviceId = 0;

static FREE_DEVICE_ID: AtomicUsize = AtomicUsize::new(1);


pub fn generate_device_id() -> DeviceId {
    FREE_DEVICE_ID.fetch_add(1, Ordering::SeqCst)
}

#[must_use]
pub fn device_map_from_slice(devices: &[Device]) -> IdToDeviceMap {
    devices
        .iter()
        .map(|device| (device.id(), device.clone()))
        .collect()
}
