use std::any::Any;

#[derive(Clone, Copy, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct NoWarn;

#[derive(Clone, Copy, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct Panic;

impl<W> Warn<W> for NoWarn { }

pub trait Warn<W> {
    fn warn(&mut self, warning: W) {
        let _ = warning;
    }
}

impl<W: Any+Send> Warn<W> for Panic {
    fn warn(&mut self, warning: W) {
        panic!(warning);
    }
}
