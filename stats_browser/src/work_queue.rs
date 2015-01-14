use std::collections::HashMap;
use std::collections::RingBuf;
use std::collections::hash_map::Entry;
use std::collections::hash_map;
use std::collections::ring_buf;
use std::default::Default;
use std::iter;
use std::num::ToPrimitive;
use std::time::duration::Duration;

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
    pub fn iter_now(&self) -> IterNow<T> {
        IterNow { iter: self.now_queue.iter() }
    }
    pub fn iter_other(&self) -> IterOther<T> {
        fn ringbuf_iter<T>(ring_buf: &RingBuf<T>) -> ring_buf::Iter<T> { ring_buf.iter() }
        let mut iters = self.other_queues.values().map(ringbuf_iter as fn(&RingBuf<Timed<T>>) -> ring_buf::Iter<Timed<T>>);
        let first = iters.next();
        IterOther {
            iter: first,
            iters: iters,
        }
    }
}

pub struct IterNow<'a,T:'a> {
    iter: ring_buf::Iter<'a,T>,
}

pub struct IterOther<'a,T:'a> {
    iter: Option<ring_buf::Iter<'a,Timed<T>>>,
    iters: iter::Map<
        &'a RingBuf<Timed<T>>,
        ring_buf::Iter<'a,Timed<T>>,
        hash_map::Values<'a,u64,RingBuf<Timed<T>>>,
        fn(&RingBuf<Timed<T>>) -> ring_buf::Iter<Timed<T>>
    >,
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl<T> Default for TimedWorkQueue<T> {
    fn default() -> TimedWorkQueue<T> {
        TimedWorkQueue::new()
    }
}

impl<'a,T> Clone for IterNow<'a,T> {
    fn clone(&self) -> IterNow<'a,T> {
        IterNow { iter: self.iter.clone() }
    }
}

impl<'a,T> Iterator for IterNow<'a,T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { self.iter.next() }
    fn size_hint(&self) -> (uint, Option<uint>) { self.iter.size_hint() }
}

impl<'a,T> ExactSizeIterator for IterNow<'a,T> { }

impl<'a,T> Iterator for IterOther<'a,T> {
    type Item = &'a Timed<T>;
    fn next(&mut self) -> Option<&'a Timed<T>> {
        loop {
            {
                let iter = match self.iter {
                    Some(ref mut i) => i,
                    None => return None,
                };
                match iter.next() {
                    Some(x) => return Some(x),
                    None => {}
                }
            }
            self.iter = self.iters.next();
        }
    }
    // TODO: implement `size_hint`
    fn size_hint(&self) -> (uint, Option<uint>) {
        let (lower, _upper) = match self.iter {
            Some(ref i) => { let (l, u) = i.size_hint(); (l, u.unwrap()) },
            None => return (0, Some(0)),
        };
        (lower, None)
    }
}
