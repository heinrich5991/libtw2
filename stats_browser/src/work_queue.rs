use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::hash_map::Entry;
use std::collections::hash_map;
use std::collections::vec_deque;
use std::default::Default;
use std::iter;

use num::ToPrimitive;

use time::Duration;
use time::Time;
use time::Timed;

#[derive(Clone)]
pub struct TimedWorkQueue<T> {
    now_queue: VecDeque<T>,
    other_queues: HashMap<u64,VecDeque<Timed<T>>>,
}

impl<T> TimedWorkQueue<T> {
    pub fn new() -> TimedWorkQueue<T> {
        TimedWorkQueue {
            now_queue: VecDeque::new(),
            other_queues: HashMap::new(),
        }
    }
    pub fn add_duration(&mut self, dur: Duration) {
        let dur_k = TimedWorkQueue::<T>::duration_to_key(dur);
        if let Entry::Vacant(v) = self.other_queues.entry(dur_k) {
            v.insert(VecDeque::new());
        }
    }
    fn duration_to_key(dur: Duration) -> u64 {
        dur.milliseconds().to_u64().expect("Expected positive duration")
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
        fn ringbuf_iter<T>(vec_deque: &VecDeque<T>) -> vec_deque::Iter<T> { vec_deque.iter() }
        let map_fn: fn(&VecDeque<Timed<T>>) -> vec_deque::Iter<Timed<T>> = ringbuf_iter;
        let mut iters = self.other_queues.values().map(map_fn);
        let first = iters.next();
        IterOther {
            iter: first,
            iters: iters,
        }
    }
}

pub struct IterNow<'a,T:'a> {
    iter: vec_deque::Iter<'a,T>,
}

pub struct IterOther<'a,T:'a> {
    iter: Option<vec_deque::Iter<'a,Timed<T>>>,
    iters: iter::Map<
        hash_map::Values<'a,u64,VecDeque<Timed<T>>>,
        fn(&VecDeque<Timed<T>>) -> vec_deque::Iter<Timed<T>>
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
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, _upper) = match self.iter {
            Some(ref i) => { let (l, u) = i.size_hint(); (l, u.unwrap()) },
            None => return (0, Some(0)),
        };
        (lower, None)
    }
}
