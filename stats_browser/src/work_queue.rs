use std::collections::HashMap;
use std::collections::RingBuf;
use std::collections::hash_map::Entry;
use std::default::Default;
use std::time::duration::Duration;
use std::num::ToPrimitive;

use time::Time;
use time::Timed;

#[derive(Clone)]
pub struct TimedWorkQueue<T> {
    now_queue: RingBuf<T>,
    other_queues: HashMap<u64,RingBuf<Timed<T>>>,
}

impl<T> TimedWorkQueue<T> {
    pub fn new() -> TimedWorkQueue<T> {
        TimedWorkQueue {
            now_queue: RingBuf::new(),
            other_queues: HashMap::new(),
        }
    }
    pub fn add_duration(&mut self, dur: Duration) {
        let dur_k = TimedWorkQueue::<T>::duration_to_key(dur);
        if let Entry::Vacant(v) = self.other_queues.entry(dur_k) {
            v.insert(RingBuf::new());
        }
    }
    fn duration_to_key(dur: Duration) -> u64 {
        dur.num_milliseconds().to_u64().expect("Expected positive duration")
    }
    pub fn push(&mut self, dur: Duration, data: T) {
        let dur_k = TimedWorkQueue::<T>::duration_to_key(dur);
        let queue = self.other_queues.get_mut(&dur_k);
        let queue = queue.expect("Need to `add_duration` before pushing with it.");
        queue.push_back(Timed::new(data, Time::now() + dur));
    }
    pub fn push_now(&mut self, data: T) {
        self.now_queue.push_back(data);
    }
    pub fn push_now_front(&mut self, data: T) {
        self.now_queue.push_front(data);
    }
    pub fn pop(&mut self) -> Option<T> {
        if let Some(data) = self.now_queue.pop_front() {
            return Some(data);
        }
        let now = Time::now();
        for (_, q) in self.other_queues.iter_mut() {
            // Only pop the first element if there actually is an element in
            // the front and it's time to process it.
            if q.front().map(|timed| timed.time <= now).unwrap_or(false) {
                return Some(q.pop_front().unwrap().data);
            }
        }
        None
    }
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl<T> Default for TimedWorkQueue<T> {
    fn default() -> TimedWorkQueue<T> {
        TimedWorkQueue::new()
    }
}
