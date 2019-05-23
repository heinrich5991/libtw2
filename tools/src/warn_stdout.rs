use std::fmt;
use warn::Warn;

pub struct Stdout;

impl<W: fmt::Debug> Warn<W> for Stdout {
    fn warn(&mut self, warning: W) {
        println!("WARN: {:?}", warning);
    }
}
