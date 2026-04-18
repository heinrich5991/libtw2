use libtw2_common::num::Cast;
use optional::Optioned;
use std::cmp;
use std::ops;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp {
    usec: u64,
}

impl Default for Timestamp {
    fn default() -> Timestamp {
        Timestamp::sentinel()
    }
}

impl Timestamp {
    pub fn sentinel() -> Timestamp {
        optional::Noned::get_none()
    }
    pub fn from_secs_since_epoch(secs: u64) -> Timestamp {
        Timestamp {
            usec: secs.checked_mul(1_000_000).unwrap(),
        }
    }
    pub fn from_usecs_since_epoch(usecs: u64) -> Timestamp {
        Timestamp { usec: usecs }
    }
    pub fn as_usecs_since_epoch(&self) -> u64 {
        self.usec
    }
}

impl ops::Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, duration: Duration) -> Timestamp {
        Timestamp::from_usecs_since_epoch(
            self.usec
                .checked_add(
                    duration
                        .as_secs()
                        .checked_mul(1_000_000)
                        .unwrap()
                        .checked_add((duration.subsec_nanos().u64() + 999) / 1_000)
                        .unwrap(),
                )
                .unwrap(),
        )
    }
}

impl optional::Noned for Timestamp {
    fn is_none(&self) -> bool {
        optional::Noned::is_none(&self.usec)
    }
    fn get_none() -> Timestamp {
        Timestamp {
            usec: optional::Noned::get_none(),
        }
    }
}

impl optional::OptEq for Timestamp {
    fn opt_eq(&self, other: &Timestamp) -> bool {
        *self == *other
    }
}

impl optional::OptOrd for Timestamp {
    fn opt_cmp(&self, other: &Timestamp) -> cmp::Ordering {
        // Make the `None` value higher than any other.
        self.cmp(other)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timeout {
    timeout: Optioned<Timestamp>,
}

impl Timeout {
    pub fn active(timestamp: Timestamp) -> Timeout {
        Timeout {
            timeout: Optioned::some(timestamp),
        }
    }
    pub fn inactive() -> Timeout {
        Timeout {
            timeout: Optioned::none(),
        }
    }
    pub fn is_active(&self) -> bool {
        self.timeout.is_some()
    }
    pub fn to_opt(self) -> Option<Timestamp> {
        self.timeout.into()
    }
    pub fn time_from(self, time: Timestamp) -> Option<Duration> {
        self.to_opt().map(|t| {
            if t > time {
                let us = t.as_usecs_since_epoch() - time.as_usecs_since_epoch();
                Duration::new(us / 1_000_000_000, (us % 1_000_000_000).assert_u32())
            } else {
                Duration::from_millis(0)
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::Timeout;
    use super::Timestamp;

    #[test]
    fn ord() {
        let t = Timeout::inactive();
        let t0 = Timeout::active(Timestamp::from_secs_since_epoch(0));
        let t1 = Timeout::active(Timestamp::from_secs_since_epoch(1));
        let t2 = Timeout::active(Timestamp::from_secs_since_epoch(2));
        assert!(t0 < t);
        assert!(t0 < t1);
        assert!(t1 < t2);
        assert!(t2 < t);
    }
}
