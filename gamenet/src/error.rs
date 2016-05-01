#[derive(Debug)]
pub struct Error {
    _unused: (),
}

impl Error {
    pub fn new() -> Error {
        Error {
            _unused: (),
        }
    }
}
