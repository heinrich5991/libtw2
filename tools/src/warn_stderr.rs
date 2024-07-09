use std::fmt;
use warn::Warn;

pub struct Stderr;

impl<W: fmt::Debug> Warn<W> for Stderr {
    fn warn(&mut self, warning: W) {
        eprintln!("WARN: {:?}", warning);
    }
}
