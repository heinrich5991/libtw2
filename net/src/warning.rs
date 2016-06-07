use WarnExt;

#[derive(Debug)]
pub struct Warning {
    _unused: (),
}

impl Warning {
    fn new() -> Warning {
        Warning {
            _unused: (),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct NoWarn;

impl Warn for NoWarn { }

pub trait Warn {
    fn warn(&mut self, warning: Warning) {
        let _ = warning;
    }
}

impl<W: Warn> WarnExt for W {
    fn warn_(&mut self) {
        self.warn(Warning::new())
    }
}
