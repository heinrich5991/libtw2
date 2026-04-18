use bitflags::bitflags;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::ops;
use std::str::FromStr;

pub const ALL: [Protocol; 2] = [Protocol::Ipv4, Protocol::Ipv6];

// Only used in-crate.
#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    Ipv4,
    Ipv6,
}

impl Protocol {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn bind_all_addr(self) -> IpAddr {
        match self {
            Protocol::Ipv4 => Ipv4Addr::new(0, 0, 0, 0).into(),
            Protocol::Ipv6 => Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Protocol::Ipv4 => "ipv4",
            Protocol::Ipv6 => "ipv6",
        }
        .fmt(f)
    }
}

pub struct ProtocolFromStrError;
impl FromStr for Protocol {
    type Err = ProtocolFromStrError;
    fn from_str(s: &str) -> Result<Protocol, ProtocolFromStrError> {
        Ok(match s {
            "ipv4" => Protocol::Ipv4,
            "ipv6" => Protocol::Ipv6,
            _ => return Err(ProtocolFromStrError),
        })
    }
}

bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq)]
    struct Flags: u8 {
        const IPV4 = 1 << 0;
        const IPV6 = 1 << 1;
    }
}

#[derive(Clone, Copy)]
pub struct Protocols(Flags);

impl Protocols {
    pub fn none() -> Protocols {
        Protocols(Flags::empty())
    }
    pub fn all() -> Protocols {
        Protocols(Flags::all())
    }
    pub fn contains(self, protocol: Protocol) -> bool {
        self.0.contains(Protocols::from(protocol).0)
    }
}

pub struct IntoIter(bitflags::iter::Iter<Flags>);

impl Iterator for IntoIter {
    type Item = Protocol;
    fn next(&mut self) -> Option<Protocol> {
        Some(match self.0.next()? {
            Flags::IPV4 => Protocol::Ipv4,
            Flags::IPV6 => Protocol::Ipv6,
            unknown => unreachable!("unknown value 0x{unknown:x}"),
        })
    }
}

impl IntoIterator for Protocols {
    type Item = Protocol;
    type IntoIter = IntoIter;
    fn into_iter(self) -> IntoIter {
        IntoIter(self.0.iter())
    }
}

impl ops::BitOr for Protocols {
    type Output = Protocols;
    fn bitor(self, other: Protocols) -> Protocols {
        Protocols(self.0 | other.0)
    }
}

impl ops::BitOrAssign for Protocols {
    fn bitor_assign(&mut self, other: Protocols) {
        *self = *self | other
    }
}

impl From<Protocol> for Protocols {
    fn from(protocol: Protocol) -> Protocols {
        Protocols(match protocol {
            Protocol::Ipv4 => Flags::IPV4,
            Protocol::Ipv6 => Flags::IPV6,
        })
    }
}

#[derive(Debug)]
pub struct ProtocolsFromStrError(Box<str>);

impl Error for ProtocolsFromStrError {}

impl fmt::Display for ProtocolsFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn from_part(part: &str) -> Option<Protocols> {
    if let Some(protocol) = Protocol::from_str(part).ok() {
        return Some(protocol.into());
    }
    // possible protocol groups here
    None
}

impl FromStr for Protocols {
    type Err = ProtocolsFromStrError;
    fn from_str(s: &str) -> Result<Protocols, ProtocolsFromStrError> {
        match s {
            "none" => return Ok(Protocols::none()),
            "all" => return Ok(Protocols::all()),
            _ => {}
        }
        let mut result = Protocols::none();
        for part in s.split(',') {
            match from_part(part) {
                Some(p) => result |= p,
                None => {
                    return Err(ProtocolsFromStrError(
                        format!("invalid protocol: {part}").into(),
                    ));
                }
            }
        }
        Ok(result)
    }
}

impl<'de> serde::Deserialize<'de> for Protocols {
    fn deserialize<D>(deserializer: D) -> Result<Protocols, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Error;
        let s: Cow<'de, str> = Cow::deserialize(deserializer)?;
        s.parse().map_err(|err| D::Error::custom(err))
    }
}
