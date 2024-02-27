#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    ControlCharacters,
    IntOutOfRange,
    InvalidIntString,
    UnexpectedEnd,
    UnknownId,
}

pub struct InvalidIntString;

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

impl From<InvalidIntString> for Error {
    fn from(_: InvalidIntString) -> Error {
        Error::InvalidIntString
    }
}

impl From<packer::UnexpectedEnd> for Error {
    fn from(_: packer::UnexpectedEnd) -> Error {
        Error::UnexpectedEnd
    }
}
