use packer;

#[derive(Debug)]
pub enum Error {
    ControlCharacters,
    IntOutOfRange,
    UnexpectedEnd,
    UnknownMessage,
}

impl From<packer::ControlCharacters> for Error {
    fn from(_: packer::ControlCharacters) -> Error {
        Error::ControlCharacters
    }
}

impl From<packer::IntOutOfRange> for Error {
    fn from(_: packer::IntOutOfRange) -> Error {
        Error::IntOutOfRange
    }
}

impl From<packer::UnexpectedEnd> for Error {
    fn from(_: packer::UnexpectedEnd) -> Error {
        Error::UnexpectedEnd
    }
}
