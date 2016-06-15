#[derive(Debug)]
pub enum Error {
    ControlCharacters,
    IntOutOfRange,
    UnexpectedEnd,
    UnknownMessage,
}

#[derive(Debug)]
pub struct IntOutOfRange;
impl From<IntOutOfRange> for Error {
    fn from(_: IntOutOfRange) -> Error {
        Error::IntOutOfRange
    }
}

#[derive(Debug)]
pub struct ControlCharacters;
impl From<ControlCharacters> for Error {
    fn from(_: ControlCharacters) -> Error {
        Error::ControlCharacters
    }
}
