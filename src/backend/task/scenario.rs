use serde::{Deserialize, Serialize};

use crate::backend::device::{DeviceId, BROADCAST_ID};
use crate::backend::mathphysics::Millisecond;

use super::Task;


type ScenarioEntry = (Millisecond, DeviceId, Task);


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Scenario(Vec<ScenarioEntry>);

impl Scenario {
    #[must_use]
    pub fn get_last_task(
        &self, 
        current_time: Millisecond, 
        destination_id: DeviceId
    ) -> Option<&Task> {
        self.0
            .iter()
            .rev()
            .find_map(|(time, device_id, task)| {
                if *time > current_time || (
                    *device_id != destination_id && *device_id != BROADCAST_ID
                ) {
                    None
                } else {
                    Some(task)
                }
            })
    }
}

impl From<&[ScenarioEntry]> for Scenario {
    fn from(scenario_entries: &[ScenarioEntry]) -> Self {
        let mut scenario = Self(scenario_entries.to_vec());

        scenario.0.sort_by_key(|(time, _, _)| *time);

        scenario
    }
}

impl<const N: usize> From<[ScenarioEntry; N]> for Scenario {
    fn from(scenario_entries: [ScenarioEntry; N]) -> Self {
        let mut scenario = Self(scenario_entries.to_vec());

        scenario.0.sort_by_key(|(time, _, _)| *time);

        scenario
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    const SOME_DEVICE_ID: DeviceId = 5;


    fn entries() -> Vec<ScenarioEntry> {
        let undefined_task = Task::Undefined;

        vec![
            (25, SOME_DEVICE_ID, undefined_task),
            (5, SOME_DEVICE_ID, undefined_task),
            (10, SOME_DEVICE_ID, undefined_task),
        ]
    }


    #[test]
    fn fail_to_get_last_task_when_current_time_is_too_early() {
        let entries = entries();

        let scenario = Scenario::from(entries.as_slice());

        assert!(scenario.get_last_task(0, SOME_DEVICE_ID).is_none());
    }

    #[test]
    fn getting_last_task() {
        let entries = entries();

        let scenario = Scenario::from(entries.as_slice());

        let last_task = *scenario
            .get_last_task(7, SOME_DEVICE_ID)
            .expect("Failed to get the last task");

        assert_eq!(last_task, entries[1].2);
    }

    #[test]
    fn getting_current_task() {
        let entries = entries();

        let scenario = Scenario::from(entries.as_slice());

        let last_task = *scenario
            .get_last_task(entries[2].0, SOME_DEVICE_ID)
            .expect("Failed to get the last task");

        assert_eq!(last_task, entries[2].2);
    }

    #[test]
    fn sort_entries_on_creation() {
        let entries = entries();

        let scenario = Scenario::from(entries.as_slice());
        let mut scenario_iter = scenario.0.into_iter();

        assert_eq!(
            entries[1].0,
            scenario_iter.next().unwrap().0
        );
        assert_eq!(
            entries[2].0,
            scenario_iter.next().unwrap().0
        );
        assert_eq!(
            entries[0].0,
            scenario_iter.next().unwrap().0
        );
    }
}
