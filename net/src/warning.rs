#[derive(Clone, Copy, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct NoWarn;

impl<W> Warn<W> for NoWarn { }

pub trait Warn<W> {
    fn warn(&mut self, warning: W) {
        let _ = warning;
    }
}
