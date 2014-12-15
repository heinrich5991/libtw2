extern crate datafile;

use datafile::DatafileBuffer;

pub mod internals;

pub struct TeeworldsMap {
    df: DatafileBuffer,
}

impl TeeworldsMap {
    pub fn from_datafile(df: DatafileBuffer) -> TeeworldsMap {
        TeeworldsMap { df: df }
    }
}
