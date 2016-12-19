use common::num::Cast;
use num::ToPrimitive;
use std::ops::Add;
use std::ops::Sub;
use std::time as std_time;

use rust_time as time;

// TODO: What happens on time overflow?
// TODO: What happens on time backward jump?

/// Duration, in milliseconds.
#[derive(Clone, Copy)]
pub struct Duration(pub i64);

impl Duration {
    pub fn milliseconds(self) -> i64 {
        let Duration(inner) = self;
        inner
    }
    pub fn to_std(self) -> std_time::Duration {
        std_time::Duration::from_millis(self.milliseconds().assert_u64())
    }
}

/// Point in time. This is strictly monotonic, but only for the runtime of the
/// program.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Time(u64); // In milliseconds.

impl Add<Duration> for Time {
    type Output = Time;
    fn add(self, rhs: Duration) -> Time {
        let Time(ms) = self;
        Time(ms + rhs.milliseconds() as u64)
    }
}

impl Sub<Time> for Time {
    type Output = Duration;
    fn sub(self, rhs: Time) -> Duration {
        let (Time(left), Time(right)) = (self, rhs);
        Duration(
            right.checked_sub(left).map(|x| x.to_i64().expect("Overflow while converting to i64"))
            .or_else(|| left.checked_sub(right).map(|x| -x.to_i64().expect("Overflow while converting to i64")))
            .expect("Overflow while subtracting")
        )
    }
}

impl Time {
    /// Returns the current `Time`.
    pub fn now() -> Time {
        Time(time::precise_time_ns() / 1_000_000)
    }
}

/// A struct holding data and a time.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Timed<T> {
    /// The data.
    pub data: T,
    /// The accompanying time.
    pub time: Time,
}

impl<T> Timed<T> {
    /// Creates a new `Timed` with the specified data and time.
    pub fn new(data: T, time: Time) -> Timed<T> {
        Timed { data: data, time: time }
    }
}

/// A struct that holds all the necessary information to throttle an action to
/// some number of actions per some amount of time.
#[derive(Copy, Clone)]
pub struct Limit {
    /// The number of remaining actions until `reset`.
    remaining: u32,
    /// The point in time when the `remaining` counter will be reset.
    reset: Time,
    /// The maximum number of actions in the time `duration`.
    max: u32,
    /// The duration after which the action counter is reset.
    duration: Duration,
}

impl Limit {
    /// Creates a new `Limit` which allows a maximum of `max` actions per
    /// `duration`.
    pub fn new(max: u32, duration: Duration) -> Limit {
        Limit {
            remaining: max,
            reset: Time::now(),
            max: max,
            duration: duration,
        }
    }
    /// Tries to acquire the `Limit` at a specific time. See `acquire` documentation.
    pub fn acquire_at(&mut self, time: Time) -> Result<(),()> {
        if time >= self.reset {
            self.remaining = self.max;
            self.reset = time + self.duration;
        }
        if self.remaining != 0 {
            self.remaining -= 1;
            Ok(())
        } else {
            Err(())
        }
    }
    /// Tries to acquire the `Limit` and consumes on action if it is within the limit.
    ///
    /// Returns `Ok(())` on success, `Err(())` on failure.
    pub fn acquire(&mut self) -> Result<(),()> {
        self.acquire_at(Time::now())
    }
}
