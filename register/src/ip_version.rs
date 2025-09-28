use std::fmt;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

pub const IP_VERSIONS: [IpVersion; 2] = [IpVersion::V4, IpVersion::V6];

#[derive(Clone, Copy)]
pub enum IpVersion {
    V4,
    V6,
}

pub struct IpVersionFromStrError;

impl IpVersion {
    pub fn index(self) -> usize {
        match self {
            IpVersion::V4 => 0,
            IpVersion::V6 => 1,
        }
    }
    pub fn bind_all(self) -> IpAddr {
        match self {
            IpVersion::V4 => Ipv4Addr::new(0, 0, 0, 0).into(),
            IpVersion::V6 => Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
        }
    }
}

impl fmt::Display for IpVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpVersion::V4 => "ipv4",
            IpVersion::V6 => "ipv6",
        }
        .fmt(f)
    }
}

impl FromStr for IpVersion {
    type Err = IpVersionFromStrError;
    fn from_str(s: &str) -> Result<IpVersion, IpVersionFromStrError> {
        Ok(match s {
            "ipv4" => IpVersion::V4,
            "ipv6" => IpVersion::V6,
            _ => return Err(IpVersionFromStrError),
        })
    }
}
