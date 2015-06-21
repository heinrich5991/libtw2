pub use datafile;

pub struct TeeworldsMap {
    df: datafile::Reader,
}

impl TeeworldsMap {
    pub fn from_datafile(df: datafile::Reader) -> TeeworldsMap {
        TeeworldsMap { df: df }
    }
}
