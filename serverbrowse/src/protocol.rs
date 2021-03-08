use arrayvec::ArrayVec;
use common::num::BeU16;
use common::num::Cast;
use common::num::LeU16;
use common::pretty;
use common;
use packer::Unpacker;
use std::default::Default;
use std::fmt;
use std::mem;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str;
use warn::Ignore;

const PLAYER_MAX_NAME_LENGTH: usize = 16-1;
const PLAYER_MAX_CLAN_LENGTH: usize = 12-1;
const MAX_CLIENTS_5:    u32 = 16;
const MAX_CLIENTS_6_64: u32 = 64;

pub const MASTERSERVER_PORT: u16 = 8300;

const HEADER_LEN: usize = 14;
pub type Header = &'static [u8; HEADER_LEN];
pub const REQUEST_LIST_5:    Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffreqt";
pub const REQUEST_LIST_6:    Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffreq2";
pub const LIST_5:            Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfflist";
pub const LIST_6:            Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfflis2";
pub const REQUEST_COUNT:     Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffcou2";
pub const COUNT:             Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffsiz2";
pub const REQUEST_INFO_5:    Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffgie2";
pub const REQUEST_INFO_6:    Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffgie3";
pub const REQUEST_INFO_6_64: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfffstd";
pub const REQUEST_INFO_6_EX: Header = b"xe\0\0\0\0\xff\xff\xff\xffgie3";
pub const INFO_5:            Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffinf2";
pub const INFO_6:            Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffinf3";
pub const INFO_6_64:         Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffdtsf";
pub const INFO_6_EX:         Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffiext";
pub const INFO_6_EX_MORE:    Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffiex+";

pub const PACKETFLAG_CONNLESS: u8 = 1 << 6;
pub const SERVERINFO_FLAG_PASSWORDED: i32 = 1 << 0;

pub const IPV4_MAPPING: [u8; 12] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
];

pub fn request_list_5() -> [u8; 14] { *REQUEST_LIST_5 }
pub fn request_list_6() -> [u8; 14] { *REQUEST_LIST_6 }

pub fn request_info_5(challenge: u8) -> [u8; 15] {
    request_info(REQUEST_INFO_5, challenge)
}
pub fn request_info_6(challenge: u8) -> [u8; 15] {
    request_info(REQUEST_INFO_6, challenge)
}
pub fn request_info_6_64(challenge: u8) -> [u8; 15] {
    request_info(REQUEST_INFO_6_64, challenge)
}
pub fn request_info_ex(challenge: u32) -> [u8; 15] {
    assert!(challenge & 0x00ff_ffff == challenge,
        "only the lower 24 bits of challenge are used");
    let mut request = [0; HEADER_LEN+1];
    request[..HEADER_LEN].copy_from_slice(REQUEST_INFO_6_EX);
    request[2] = ((challenge & 0x00ff_0000) >> 16) as u8;
    request[3] = ((challenge & 0x0000_ff00) >> 8) as u8;
    request[HEADER_LEN] = ((challenge & 0x0000_00ff) >> 0) as u8;
    request
}

pub fn request_count() -> [u8; 14] { *REQUEST_COUNT }

fn request_info(header: Header, challenge: u8) -> [u8; 15] {
    let mut request = [0; HEADER_LEN+1];
    request[..HEADER_LEN].copy_from_slice(header);
    request[HEADER_LEN] = challenge;
    request
}



#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClientInfo {
    pub name: ArrayVec<[u8; PLAYER_MAX_NAME_LENGTH]>,
    pub clan: ArrayVec<[u8; PLAYER_MAX_CLAN_LENGTH]>,
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}

impl fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?} {:?} {:?} {:?}",
            pretty::AlmostString::new(&self.name),
            pretty::AlmostString::new(&self.clan),
            self.country,
            self.score,
            self.is_player,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ServerInfoVersion {
    V5,
    V6,
    V664,
    V6Ex,
    V7,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ReceivedServerInfoVersion {
    Normal(ServerInfoVersion),
    V6ExMore,
}

impl From<ReceivedServerInfoVersion> for ServerInfoVersion {
    fn from(version: ReceivedServerInfoVersion) -> ServerInfoVersion {
        use self::ReceivedServerInfoVersion::*;
        match version {
            Normal(n) => n,
            V6ExMore => ServerInfoVersion::V6Ex,
        }
    }
}

impl ReceivedServerInfoVersion {
    fn is_normal(self) -> bool {
        use self::ReceivedServerInfoVersion::*;
        match self {
            Normal(_) => true,
            _ => false,
        }
    }
}

impl ServerInfoVersion {
    pub fn max_clients(self) -> Option<u32> {
        Some(match self {
            ServerInfoVersion::V5       => MAX_CLIENTS_5,
            ServerInfoVersion::V6       => MAX_CLIENTS_5,
            ServerInfoVersion::V664     => MAX_CLIENTS_6_64,
            ServerInfoVersion::V6Ex     => return None,
            ServerInfoVersion::V7       => MAX_CLIENTS_5,
        })
    }
    pub fn clients_per_packet(self) -> Option<u32> {
        Some(match self {
            ServerInfoVersion::V5       => 16,
            ServerInfoVersion::V6       => 16,
            ServerInfoVersion::V664     => 24,
            ServerInfoVersion::V6Ex     => return None,
            ServerInfoVersion::V7       => 16,
        })
    }
    pub fn has_hostname(self) -> bool {
        self >= ServerInfoVersion::V7
    }
    pub fn has_progression(self) -> bool {
        self == ServerInfoVersion::V5
    }
    pub fn has_skill_level(self) -> bool {
        self >= ServerInfoVersion::V7
    }
    pub fn has_offset(self) -> bool {
        self == ServerInfoVersion::V664
    }
    pub fn has_extended_player_info(self) -> bool {
        self >= ServerInfoVersion::V6
    }
    pub fn has_extended_map_info(self) -> bool {
        self == ServerInfoVersion::V6Ex
    }
    pub fn has_extra_info(self) -> bool {
        self == ServerInfoVersion::V6Ex
    }
}

impl Default for ServerInfoVersion { fn default() -> ServerInfoVersion { ServerInfoVersion::V5 } }

#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct ServerInfo {
    pub info_version: ServerInfoVersion,
    pub token: i32,
    pub version: ArrayVec<[u8; 32]>,
    pub name: ArrayVec<[u8; 64]>,
    pub hostname: Option<ArrayVec<[u8; 64]>>,
    pub map: ArrayVec<[u8; 32]>,
    pub map_crc: Option<u32>,
    pub map_size: Option<u32>,
    pub game_type: ArrayVec<[u8; 32]>,
    pub flags: i32,
    pub progression: Option<i32>,
    pub skill_level: Option<i32>,
    pub num_players: i32,
    pub max_players: i32,
    pub num_clients: i32,
    pub max_clients: i32,
    pub clients: Vec<ClientInfo>,
}

impl ServerInfo {
    pub fn sort_clients(&mut self) {
        self.clients.sort();
    }
}

impl fmt::Debug for ServerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}/{:?} {:?}/{:?}: {:?}",
            self.info_version,
            self.token,
            pretty::AlmostString::new(&self.version),
            pretty::AlmostString::new(&self.name),
            self.hostname.as_ref().map(|x| pretty::AlmostString::new(x)),
            pretty::AlmostString::new(&self.map),
            pretty::AlmostString::new(&self.game_type),
            self.flags,
            self.progression,
            self.skill_level,
            self.num_players,
            self.max_players,
            self.num_clients,
            self.max_clients,
            self.clients,
        )
    }
}

#[derive(Clone, Debug)]
pub struct PartialServerInfo {
    info: ServerInfo,
    received: u64,
}

#[derive(Clone, Debug)]
pub enum MergeError {
    DifferingTokens,
    DifferingVersions,
    NotMultipartVersion,
    OverlappingInfos,
}

impl PartialServerInfo {
    fn new() -> PartialServerInfo {
        PartialServerInfo {
            info: Default::default(),
            received: Default::default(),
        }
    }
    // TODO: What to do when the infos don't match?
    // Currently the other info is just ignored.
    pub fn merge(&mut self, mut other: PartialServerInfo) -> Result<(),MergeError> {
        if self.info.token != other.info.token {
            return Err(MergeError::DifferingTokens);
        }
        if self.info.info_version != other.info.info_version {
            return Err(MergeError::DifferingVersions);
        }
        if self.info.info_version != ServerInfoVersion::V664
            && self.info.info_version != ServerInfoVersion::V6Ex
        {
            return Err(MergeError::NotMultipartVersion);
        }
        if self.received & other.received == other.received {
            // We already have that server info.
            // TODO: What to do if it doesn't match?
            return Ok(());
        }
        if self.received & other.received != 0 {
            return Err(MergeError::OverlappingInfos);
        }
        if self.info.info_version == ServerInfoVersion::V6Ex &&
            self.received & 1 == 0
        {
            mem::swap(self, &mut other);
        }
        self.info.clients.extend(other.info.clients.into_iter());

        Ok(())
    }
    pub fn get_info(&mut self) -> Option<&ServerInfo> {
        if self.info.clients.len().assert_i32() != self.info.num_clients {
            return None;
        }
        self.info.clients.sort();
        Some(&self.info)
    }
}

fn debug_parse_fail(help: &str) -> Option<PartialServerInfo> {
    fn debug_parse_fail_impl(help: &str) {
        debug!("server info parsing failed at {}", help);
        let _ = help;
    }
    debug_parse_fail_impl(help);
    None
}

fn parse_server_info<RI,RS>(
    unpacker: &mut Unpacker,
    read_int: RI,
    read_str: RS,
    received_version: ReceivedServerInfoVersion,
) -> Option<PartialServerInfo>
    where RI: FnMut(&mut Unpacker) -> Option<i32>,
          RS: for<'a> FnMut(&mut Unpacker<'a>) -> Option<&'a [u8]>,
{
    use self::debug_parse_fail as fail;

    let mut read_int = read_int;
    let mut read_str = read_str;
    let mut result = PartialServerInfo::new();

    macro_rules! int { ($cause:expr) => {
        unwrap_or_return!(read_int(unpacker), fail($cause));
    } }

    macro_rules! str { ($cause:expr) => {
        unwrap_or_return!(read_str(unpacker), fail($cause))
            .iter().cloned().collect();
    } }

    let version: ServerInfoVersion = received_version.into();

    {
        let i = &mut result.info;
        i.info_version = version;
        i.token = int!("token");
        let packet_no;
        let offset;
        if !received_version.is_normal() {
            packet_no = int!("packet_no");
            if packet_no < 1 || packet_no > 64 {
                return fail("packet_no sanity check");
            }
            offset = 0;
        } else {
            packet_no = 0;
            i.version = str!("version");
            i.name = str!("name");
            if version.has_hostname() {
                i.hostname = Some(str!("hostname"));
            } else {
                i.hostname = None;
            }
            i.map         = str!("map");
            if version.has_extended_map_info() {
                i.map_crc = Some(int!("map_crc") as u32);
                let map_size = int!("map_size");
                if map_size < 0 {
                    return fail("map_size sanity check");
                }
                i.map_size = Some(map_size.assert_u32());
            } else {
                i.map_crc = None;
                i.map_size = None;
            }
            i.game_type   = str!("game_type");
            i.flags       = int!("flags");
            if version.has_progression() {
                i.progression = Some(int!("progression"));
            } else {
                i.progression = None;
            }
            if version.has_skill_level() {
                i.skill_level = Some(int!("skill_level"));
            } else {
                i.skill_level = None;
            }
            i.num_players = int!("num_players");
            i.max_players = int!("max_players");
            if version.has_extended_player_info() {
                i.num_clients = int!("num_clients");
                i.max_clients = int!("max_clients");
            } else {
                i.num_clients = i.num_players;
                i.max_clients = i.max_players;
            }
            let raw_offset;
            if version.has_offset() {
                raw_offset = int!("offset");
            } else {
                raw_offset = 0;
            }
            if i.num_clients < 0 || i.num_clients > i.max_clients
                || i.max_clients < 0
                || version.max_clients().map(|m| i.max_clients > m.assert_i32()).unwrap_or(false)
                || i.num_players < 0 || i.num_players > i.num_clients
                || i.max_players < 0 || i.max_players > i.max_clients
            {
                return fail("count sanity check");
            }
            offset = unwrap_or_return!(raw_offset.try_u32(), fail("offset sanity check"));
        }
        if version.has_extra_info() {
            let _: ArrayVec<[u8; 0]> = str!("extra_info");
        }

        if version == ServerInfoVersion::V6Ex {
            result.received |= 1 << packet_no;
        }

        for j in offset.. {
            let name = match read_str(unpacker) {
                Some(n) => n.iter().cloned().collect(),
                None => break,
            };
            let clan;
            let country;
            if version.has_extended_player_info() {
                clan    = str!("client_clan");
                country = int!("client_country");
            } else {
                clan    = Default::default();
                country = -1;
            }
            let score = int!("client_score");
            let is_player;
            if version.has_extended_player_info() {
                is_player = int!("client_is_player");
            } else {
                is_player = 1;
            }
            if version.has_extra_info() {
                let _: ArrayVec<[u8; 0]> = str!("extra_info");
            }
            if version == ServerInfoVersion::V664 {
                if j > MAX_CLIENTS_6_64 {
                    continue;
                } else {
                    result.received |= 1 << j;
                }
            }
            i.clients.push(ClientInfo {
                name: name,
                clan: clan,
                country: country,
                score: score,
                is_player: is_player,
            });
        }
    }
    Some(result)
}

fn info_read_int_v5(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_string()
        .ok()
        .and_then(|x| str::from_utf8(x).ok())
        .and_then(|x| x.parse().ok())
}

fn info_read_int_v7(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_int(&mut Ignore).ok()
}

fn info_read_str<'a>(unpacker: &mut Unpacker<'a>) -> Option<&'a [u8]> {
    unpacker.read_string().ok()
}

impl<'a> Info5Response<'a> {
    pub fn parse(self) -> Option<ServerInfo> {
        let Info5Response(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ReceivedServerInfoVersion::Normal(ServerInfoVersion::V5),
        ).map(|mut raw| { raw.info.sort_clients(); raw.info })
    }
}

impl<'a> Info6Response<'a> {
    fn parse_impl<RI,RS>(self,
        read_int: RI,
        read_str: RS,
        version: ServerInfoVersion,
    ) -> Option<ServerInfo>
    where RI: FnMut(&mut Unpacker) -> Option<i32>,
          RS: for<'b> FnMut(&mut Unpacker<'b>) -> Option<&'b [u8]>,
    {
        let Info6Response(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        let version = ReceivedServerInfoVersion::Normal(version);
        parse_server_info(&mut unpacker, read_int, read_str, version)
            .map(|mut raw| { raw.info.sort_clients(); raw.info })
    }
    pub fn parse_v6(self) -> Option<ServerInfo> {
        self.parse_impl(info_read_int_v5, info_read_str, ServerInfoVersion::V6)
    }
    pub fn parse_v7(self) -> Option<ServerInfo> {
        self.parse_impl(info_read_int_v7, info_read_str, ServerInfoVersion::V7)
    }
    pub fn parse(self) -> Option<ServerInfo> {
        Info6Response::parse_v6(self).or_else(|| Info6Response::parse_v7(self))
    }
}

impl<'a> Info664Response<'a> {
    pub fn parse(self) -> Option<PartialServerInfo> {
        let Info664Response(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ReceivedServerInfoVersion::Normal(ServerInfoVersion::V664),
        )
    }
}

impl<'a> Info6ExResponse<'a> {
    pub fn parse(self) -> Option<PartialServerInfo> {
        let Info6ExResponse(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ReceivedServerInfoVersion::Normal(ServerInfoVersion::V6Ex),
        )
    }
}

impl<'a> Info6ExMoreResponse<'a> {
    pub fn parse(self) -> Option<PartialServerInfo> {
        let Info6ExMoreResponse(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ReceivedServerInfoVersion::V6ExMore,
        )
    }
}

#[derive(Copy, Clone)] pub struct Info5Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct Info6Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct Info664Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct Info6ExResponse<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct Info6ExMoreResponse<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct CountResponse(pub u16);
#[derive(Copy, Clone)] pub struct List5Response<'a>(pub &'a [Addr5Packed]);
#[derive(Copy, Clone)] pub struct List6Response<'a>(pub &'a [Addr6Packed]);
#[derive(Copy, Clone)] pub struct PongResponse(pub i32);

#[derive(Copy, Clone)]
pub enum Response<'a> {
    List5(List5Response<'a>),
    List6(List6Response<'a>),
    Count(CountResponse),
    Info5(Info5Response<'a>),
    Info6(Info6Response<'a>),
    Info664(Info664Response<'a>),
    Info6Ex(Info6ExResponse<'a>),
    Info6ExMore(Info6ExMoreResponse<'a>),
    Pong(PongResponse),
}

fn parse_list5(data: &[u8]) -> &[Addr5Packed] {
    let remainder = data.len() % mem::size_of::<Addr5Packed>();
    if remainder != 0 {
        warn!("parsing overlong list5");
    }
    let data = &data[..data.len() - remainder];
    unsafe { common::slice::transmute(data) }
}

fn parse_list6(data: &[u8]) -> &[Addr6Packed] {
    let remainder = data.len() % mem::size_of::<Addr6Packed>();
    if remainder != 0 {
        warn!("parsing overlong list5");
    }
    let data = &data[..data.len() - remainder];
    unsafe { common::slice::transmute(data) }
}

fn parse_count(data: &[u8]) -> Option<u16> {
    if data.len() < 2 {
        return None;
    }
    if data.len() > 2 {
        warn!("parsing overlong count");
    }
    Some(((data[0] as u16) << 8) | (data[1] as u16))
}

pub fn parse_response(data: &[u8]) -> Option<Response> {
    if data.len() < HEADER_LEN {
        return None;
    }
    if data[0] & PACKETFLAG_CONNLESS == 0 {
        return None;
    }
    let (header, data) = data.split_at(HEADER_LEN);
    let mut header: [u8; HEADER_LEN] = *unsafe { &*(header.as_ptr() as *const [u8; HEADER_LEN]) };
    for b in &mut header[..6] {
        *b = 0xff;
    }
    match &header {
        LIST_5 => Some(Response::List5(List5Response(parse_list5(data)))),
        LIST_6 => Some(Response::List6(List6Response(parse_list6(data)))),
        INFO_5 => Some(Response::Info5(Info5Response(data))),
        INFO_6 => Some(Response::Info6(Info6Response(data))),
        INFO_6_64 => Some(Response::Info664(Info664Response(data))),
        INFO_6_EX => Some(Response::Info6Ex(Info6ExResponse(data))),
        INFO_6_EX_MORE => Some(Response::Info6ExMore(Info6ExMoreResponse(data))),
        COUNT => parse_count(data).map(|x| Response::Count(CountResponse(x))),
        _ => None,
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl IpAddr {
    fn new_v4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }
    fn new_v6(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(a, b, c, d, e, f, g, h))
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Addr {
    pub ip_address: IpAddr,
    pub port: u16,
}

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip_address, self.port)
    }
}

impl fmt::Debug for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpAddr::V4(i) => write!(f, "{}", i),
            IpAddr::V6(i) => write!(f, "[{}]", i),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Addr5Packed {
    ip_address: [u8; 4],
    port: LeU16,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Addr6Packed {
    ip_address: [u8; 16],
    port: BeU16,
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[test] fn check_alignment_addr5_packed() { assert_eq!(mem::align_of::<Addr5Packed>(), 1); }

impl Addr5Packed {
    pub fn unpack(self) -> Addr {
        let Addr5Packed { ip_address, port } = self;
        Addr {
            ip_address: IpAddr::new_v4(ip_address[0], ip_address[1], ip_address[2], ip_address[3]),
            port: port.to_u16(),
        }
    }
}

#[test] fn check_alignment_addr6_packed() { assert_eq!(mem::align_of::<Addr6Packed>(), 1); }

impl Addr6Packed {
    pub fn unpack(self) -> Addr {
        let Addr6Packed { ip_address, port } = self;
        let (maybe_ipv4_mapping, ipv4_address) = ip_address.split_at(IPV4_MAPPING.len());
        let new_address = if maybe_ipv4_mapping != IPV4_MAPPING {
            let ip_address: [BeU16; 8] = unsafe { mem::transmute(ip_address) };
            IpAddr::new_v6(
                ip_address[0].to_u16(),
                ip_address[1].to_u16(),
                ip_address[2].to_u16(),
                ip_address[3].to_u16(),
                ip_address[4].to_u16(),
                ip_address[5].to_u16(),
                ip_address[6].to_u16(),
                ip_address[7].to_u16(),
            )
        } else {
            IpAddr::new_v4(ipv4_address[0], ipv4_address[1], ipv4_address[2], ipv4_address[3])
        };
        Addr {
            ip_address: new_address,
            port: port.to_u16(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use super::ClientInfo;
    use super::Info6ExMoreResponse;
    use super::Info6ExResponse;
    use super::Info6Response;
    use super::ServerInfo;
    use super::ServerInfoVersion;

    fn b<FI: FromIterator<u8>>(s: &str) -> FI {
        s.as_bytes().iter().cloned().collect()
    }

    #[test]
    fn parse_info_v6_real_world() {
        let info_raw = b"0\x000.6.4, 11.2.1\x00DDNet RUS - Moderate [DDraceNetwork] [0/64]\x00Sunreal\x00DDraceNetwork\x000\x000\x0016\x000\x0016\x00";
        let info = ServerInfo {
            info_version: ServerInfoVersion::V6,
            token: 0,
            version: b("0.6.4, 11.2.1"),
            name: b("DDNet RUS - Moderate [DDraceNetwork] [0/64]"),
            hostname: None,
            map: b("Sunreal"),
            map_crc: None,
            map_size: None,
            game_type: b("DDraceNetwork"),
            flags: 0,
            progression: None,
            skill_level: None,
            num_players: 0,
            max_players: 16,
            num_clients: 0,
            max_clients: 16,
            clients: vec![],
        };
        assert_eq!(Info6Response(info_raw).parse(), Some(info));
    }

    #[test]
    fn parse_info_v6() {
        let info_raw = b"1\0two\0three\0four\0five\06\01\02\02\03\0seven\0eight\0-1\09\010\0eleven\0twelve\013\014\015\0";
        let info = ServerInfo {
            info_version: ServerInfoVersion::V6,
            token: 1,
            version: b("two"),
            name: b("three"),
            hostname: None,
            map: b("four"),
            map_crc: None,
            map_size: None,
            game_type: b("five"),
            flags: 6,
            progression: None,
            skill_level: None,
            num_players: 1,
            max_players: 2,
            num_clients: 2,
            max_clients: 3,
            clients: vec![
                ClientInfo {
                    name: b("eleven"),
                    clan: b("twelve"),
                    country: 13,
                    score: 14,
                    is_player: 15,
                },
                ClientInfo {
                    name: b("seven"),
                    clan: b("eight"),
                    country: -1,
                    score: 9,
                    is_player: 10,
                },
            ],
        };
        assert_eq!(Info6Response(info_raw).parse(), Some(info));
    }

    #[test]
    fn parse_info_v6_ex() {
        let info_raw_p0 = b"86536\0version\0name\0map\06277493\0627272\0gametype\035247\03\06\09\012\0\0player8\0clan8\08\088\01\0\0player3\0clan3\03\033\01\0\0player1\0clan1\01\011\00\0\0";
        let info_raw_p1 = b"86536\01\0\0player4\0clan4\04\044\00\0\0player6\0clan6\06\066\00\0\0player5\0clan5\05\055\00\0\0";
        let info_raw_p2 = b"86536\01\0\0player9\0clan9\09\099\00\0\0player7\0clan7\07\077\01\0\0player2\0clan2\02\022\00\0\0";
        let wanted = ServerInfo {
            info_version: ServerInfoVersion::V6Ex,
            token: 86536,
            version: b("version"),
            name: b("name"),
            hostname: None,
            map: b("map"),
            map_crc: Some(6277493),
            map_size: Some(627272),
            game_type: b("gametype"),
            flags: 35247,
            progression: None,
            skill_level: None,
            num_players: 3,
            max_players: 6,
            num_clients: 9,
            max_clients: 12,
            clients: vec![
                ClientInfo { name: b("player1"), clan: b("clan1"), country: 1, score: 11, is_player: 0 },
                ClientInfo { name: b("player2"), clan: b("clan2"), country: 2, score: 22, is_player: 0 },
                ClientInfo { name: b("player3"), clan: b("clan3"), country: 3, score: 33, is_player: 1 },
                ClientInfo { name: b("player4"), clan: b("clan4"), country: 4, score: 44, is_player: 0 },
                ClientInfo { name: b("player5"), clan: b("clan5"), country: 5, score: 55, is_player: 0 },
                ClientInfo { name: b("player6"), clan: b("clan6"), country: 6, score: 66, is_player: 0 },
                ClientInfo { name: b("player7"), clan: b("clan7"), country: 7, score: 77, is_player: 1 },
                ClientInfo { name: b("player8"), clan: b("clan8"), country: 8, score: 88, is_player: 1 },
                ClientInfo { name: b("player9"), clan: b("clan9"), country: 9, score: 99, is_player: 0 },
            ],
        };
        let info_p0 = Info6ExResponse(info_raw_p0).parse().unwrap();
        let info_p1 = Info6ExMoreResponse(info_raw_p1).parse().unwrap();
        let info_p2 = Info6ExMoreResponse(info_raw_p2).parse().unwrap();
        println!("{:?}", info_p0);
        println!("{:?}", info_p1);
        println!("{:?}", info_p2);
        println!("merging");
        let mut info = info_p0;
        info.merge(info_p1).unwrap();
        println!("{:?}", info);
        info.merge(info_p2).unwrap();
        println!("{:?}", info);
        assert_eq!(info.get_info(), Some(&wanted));

    }
}
