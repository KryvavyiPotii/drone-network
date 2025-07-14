use serde::{Deserialize, Serialize};

use crate::backend::device::{DeviceId, IdToDelayMap, BROADCAST_ID}; 
use crate::backend::mathphysics::Millisecond;

use super::Signal;


// The first element - time of signal creation.
// The second element - the signal.
// The third element - delays of sending the signal to devices.
type SignalQueueEntry = (Millisecond, Signal, IdToDelayMap);


fn any_delay_for(
    device_id: DeviceId, 
    delay_map: &IdToDelayMap
) -> Millisecond {
    if let Some(delay) = delay_map.get(&device_id) {
        return *delay;
    }
    if let Some(broadcast_delay) = delay_map.get(&BROADCAST_ID) {
        return *broadcast_delay;
    }

    0
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SignalQueue(Vec<SignalQueueEntry>);

impl SignalQueue {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
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
    pub fn get_current_signals_for(
        &self, 
        destination_id: DeviceId,
        current_time: Millisecond, 
    ) -> Vec<&Signal> {
        self.0
            .iter()
            .filter_map(|(time, signal, delay_map)| {
                let delay = any_delay_for(destination_id, delay_map);

                if current_time == time + delay 
                    && signal.destination_id() == destination_id 
                {
                    Some(signal)
                } else {
                    None
                }
            })
            .collect()
    }
   
    pub fn add_entry(
        &mut self, 
        time: Millisecond,
        signal: Signal, 
        delay_map: IdToDelayMap
    ) {
        self.0.push((time, signal, delay_map));
        self.0.sort_by_key(|(time, _, _)| *time);
    }

    pub fn remove_old_signals(&mut self, current_time: Millisecond) {
        self.0.retain(|(time, _, delay_map)| {
            let longest_delay = delay_map
                .values()
                .max()
                .unwrap_or(&0);

            // We assume that the signal processing is finished if it was 
            // processed by a device with the longest delay. 
            current_time < time + longest_delay
        });
    }
}


#[cfg(test)]
mod tests {
    use crate::backend::device::DeviceId;
    use crate::backend::mathphysics::Megahertz;
    use crate::backend::signal::BLACK_SIGNAL_LEVEL;

    use super::*;


    const SOME_ID: DeviceId         = 5;
    const SOME_FREQUENCY: Megahertz = 2_000;


    fn time_and_signals() -> Vec<(Millisecond, Signal)> {
        let signal1 = Signal::new(
            SOME_ID,
            SOME_ID,
            None,
            SOME_FREQUENCY,
            BLACK_SIGNAL_LEVEL,
        );
        let signal2 = Signal::new(
            SOME_ID,
            SOME_ID,
            None,
            SOME_FREQUENCY,
            BLACK_SIGNAL_LEVEL,
        );
        let signal3 = Signal::new(
            SOME_ID,
            SOME_ID,
            None,
            SOME_FREQUENCY,
            BLACK_SIGNAL_LEVEL,
        );

        vec![
            (25, signal1), 
            (5, signal2), 
            (10, signal3)
        ]
    }


    #[test]
    fn removing_older_signals() {
        let time_and_signals = time_and_signals();

        let mut signal_queue = SignalQueue(
            time_and_signals
                .iter()
                .map(|(time, signal)| (*time, *signal, IdToDelayMap::new()))
                .collect()
        );

        signal_queue.remove_old_signals(10);

        assert_eq!(signal_queue.len(), 1);
        assert_eq!(signal_queue.0[0].1, time_and_signals[0].1);
    }
    
    #[test]
    fn sort_signals_while_adding() {
        let time_and_signals = time_and_signals();
        let mut signal_queue = SignalQueue::new();

        for (time, signal) in &time_and_signals {
            signal_queue.add_entry(*time, *signal, IdToDelayMap::default());
        }

        let mut queue_iter = signal_queue.0.into_iter();

        assert_eq!(
            time_and_signals[1].0,
            queue_iter.next().unwrap().0
        );
        assert_eq!(
            time_and_signals[2].0,
            queue_iter.next().unwrap().0
        );
        assert_eq!(
            time_and_signals[0].0,
            queue_iter.next().unwrap().0
        );
    }
}
