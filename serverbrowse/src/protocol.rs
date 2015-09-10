use common;
use common::num::BeU16;
use common::num::LeU16;

use arrayvec::ArrayVec;
use num::ToPrimitive;
use std::default::Default;
use std::fmt;
use std::mem;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::slice;
use std::str;

const MAX_CLIENTS:      u32 = 64;
const MAX_CLIENTS_5:    u32 = 16;
const MAX_CLIENTS_6_64: u32 = 64;

pub const MASTERSERVER_PORT: u16 = 8300;

/// Zero-terminated byte sequence.
pub struct ZBytes([u8]);

impl ZBytes {
    pub fn check_bytes(bytes: &[u8]) -> bool {
        let mut iter = bytes.iter();
        // Check that the last byte exists and is zero.
        if *unwrap_or_return!(iter.next_back(), false) != 0 {
            return false;
        }
        for &b in iter {
            if b == 0 {
                return false;
            }
        }
        true
    }
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &ZBytes {
        mem::transmute(bytes)
    }
    pub fn from_bytes(bytes: &[u8]) -> Option<&ZBytes> {
        if !ZBytes::check_bytes(bytes) {
            return None;
        }
        Some(unsafe { ZBytes::from_bytes_unchecked(bytes) })
    }
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut ZBytes {
        mem::transmute(bytes)
    }
    pub fn from_bytes_mut(bytes: &mut [u8]) -> Option<&mut ZBytes> {
        if !ZBytes::check_bytes(bytes) {
            return None;
        }
        Some(unsafe { ZBytes::from_bytes_unchecked_mut(bytes) })
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    // Cannot be public due to invariant that needs to be checked.
    unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
    pub fn slice_from(&self, begin: usize) -> &ZBytes {
        assert!(begin < self.as_bytes().len(), "cannot slice nul byte away");
        unsafe { ZBytes::from_bytes_unchecked(&self.as_bytes()[begin..]) }
    }
    pub fn slice_from_mut(&mut self, begin: usize) -> &mut ZBytes {
        assert!(begin < self.as_bytes().len(), "cannot slice nul byte away");
        unsafe { ZBytes::from_bytes_unchecked_mut(&mut self.as_bytes_mut()[begin..]) }
    }
    pub fn as_bytes_without_nul(&self) -> &[u8] {
        let len = self.as_bytes().len();
        &self.as_bytes()[..len - 1]
    }
    pub fn as_bytes_without_nul_mut(&mut self) -> &mut [u8] {
        let len = self.as_bytes().len();
        unsafe { &mut self.as_bytes_mut()[..len - 1] }
    }
}

// Format: ESDDDDDD EDDDDDDD EDDDDDDD EDDDDDDD ...
// E - Extend
// S - Sign
// D - Digit
pub fn read_int(iter: &mut slice::Iter<u8>) -> Option<i32> {
    let mut result = 0;

    let mut src = *unwrap_or_return!(iter.next(), None);
    let sign = ((src >> 6) & 1) as i32;

    result |= (src & 0b0011_1111) as i32;

    for i in 0..4 {
        if src & 0b1000_0000 == 0 {
            break;
        }
        src = *unwrap_or_return!(iter.next(), None);
        result |= ((src & 0b0111_1111) as i32) << (6 + 7 * i as usize);
    }

    result ^= -sign;

    Some(result)
}

pub fn read_string<'a>(iter: &mut slice::Iter<'a,u8>) -> Option<&'a ZBytes> {
    let mut first_byte = None;
    // `by_ref` is needed as the iterator is silently copied otherwise.
    for (i, c) in iter.by_ref().enumerate() {
        if let None = first_byte {
            first_byte = Some(c);
        }
        if *c == 0 {
            let slice = unsafe { slice::from_raw_parts(first_byte.unwrap(), i + 1) };
            return Some(unsafe { ZBytes::from_bytes_unchecked(slice) });
        }
    }
    None
}

pub struct Unpacker<'a> {
    iter: slice::Iter<'a,u8>,
}

impl<'a> Unpacker<'a> {
    pub fn from_iter(iter: slice::Iter<'a,u8>) -> Unpacker<'a> {
        Unpacker { iter: iter }
    }
    pub fn from_slice(slice: &'a [u8]) -> Unpacker<'a> { Unpacker::from_iter(slice.iter()) }
    pub fn read_int(&mut self) -> Option<i32> {
        read_int(&mut self.iter)
    }
    pub fn read_string(&mut self) -> Option<&'a ZBytes> {
        read_string(&mut self.iter)
    }
    pub fn is_end(&self) -> bool {
        self.iter.len() == 0
    }
}

// TODO: better literals? :(
const HEADER_LEN: usize = 14;
pub type Header = [u8; HEADER_LEN];
pub const REQUEST_LIST_5:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'r', b'e', b'q', b't']; // "reqt"
pub const REQUEST_LIST_6:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'r', b'e', b'q', b'2']; // "req2"
pub const LIST_5:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'l', b'i', b's', b't']; // "list"
pub const LIST_6:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'l', b'i', b's', b'2']; // "lis2"
pub const REQUEST_COUNT:     Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'c', b'o', b'u', b'2']; // "cou2"
pub const COUNT:             Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b's', b'i', b'z', b'2']; // "siz2"
pub const REQUEST_INFO_5:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'g', b'i', b'e', b'2']; // "gie2"
pub const REQUEST_INFO_6:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'g', b'i', b'e', b'3']; // "gie3"
pub const REQUEST_INFO_6_64: Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'f', b's', b't', b'd']; // "fstd"
pub const INFO_5:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'i', b'n', b'f', b'2']; // "inf2"
pub const INFO_6:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'i', b'n', b'f', b'3']; // "inf3"
pub const INFO_6_64:         Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, b'd', b't', b's', b'f']; // "dtsf"

pub const IPV4_MAPPING: [u8; 12] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff];

pub fn request_list_5<T,S>(send: S) -> T where S: FnOnce(&[u8]) -> T { send(&REQUEST_LIST_5) }
pub fn request_list_6<T,S>(send: S) -> T where S: FnOnce(&[u8]) -> T { send(&REQUEST_LIST_6) }

pub fn request_info_5   <T,S>(send: S) -> T where S: FnOnce(&[u8]) -> T { request_info_num_5   (0, send) }
pub fn request_info_6   <T,S>(send: S) -> T where S: FnOnce(&[u8]) -> T { request_info_num_6   (0, send) }
pub fn request_info_6_64<T,S>(send: S) -> T where S: FnOnce(&[u8]) -> T { request_info_num_6_64(0, send) }
pub fn request_info_num_5   <T,S>(num: u8, send: S) -> T where S: FnOnce(&[u8]) -> T { request_info_num(&REQUEST_INFO_5,    num, send) }
pub fn request_info_num_6   <T,S>(num: u8, send: S) -> T where S: FnOnce(&[u8]) -> T { request_info_num(&REQUEST_INFO_6,    num, send) }
pub fn request_info_num_6_64<T,S>(num: u8, send: S) -> T where S: FnOnce(&[u8]) -> T { request_info_num(&REQUEST_INFO_6_64, num, send) }

pub fn request_count<T,S>(send: S) -> T where S: FnOnce(&[u8]) -> T { send(&REQUEST_COUNT) }

fn request_info_num<T,S>(header: &Header, num: u8, send: S) -> T where S: FnOnce(&[u8]) -> T {
    let mut request = [0; HEADER_LEN+1];
    request[HEADER_LEN] = num;
    for (i, &v) in header.iter().enumerate() { request[i] = v; }
    send(&request)
}



#[allow(missing_copy_implementations)]
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClientInfo {
    pub name: ArrayVec<[u8; 16]>,
    pub clan: ArrayVec<[u8; 16]>,
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}

impl fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?} {:?} {:?} {:?}",
            String::from_utf8_lossy(&self.name),
            String::from_utf8_lossy(&self.clan),
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
    V7,
}

impl ServerInfoVersion {
    pub fn max_clients(self) -> u32 {
        match self {
            ServerInfoVersion::V5   => MAX_CLIENTS_5,
            ServerInfoVersion::V6   => MAX_CLIENTS_5,
            ServerInfoVersion::V664 => MAX_CLIENTS_6_64,
            ServerInfoVersion::V7   => MAX_CLIENTS_5,
        }
    }
    pub fn clients_per_packet(self) -> u32 {
        match self {
            ServerInfoVersion::V5   => 16,
            ServerInfoVersion::V6   => 16,
            ServerInfoVersion::V664 => 24,
            ServerInfoVersion::V7   => 16,
        }
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
    pub game_type: ArrayVec<[u8; 32]>,
    pub flags: i32,
    pub progression: Option<i32>,
    pub skill_level: Option<i32>,
    pub num_players: i32,
    pub max_players: i32,
    pub num_clients: i32,
    pub max_clients: i32,
    pub clients: ArrayVec<[ClientInfo; MAX_CLIENTS as usize]>,
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
            String::from_utf8_lossy(&self.version),
            String::from_utf8_lossy(&self.name),
            self.hostname.as_ref().map(|x| String::from_utf8_lossy(x).into_owned()),
            String::from_utf8_lossy(&self.map),
            String::from_utf8_lossy(&self.game_type),
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

#[derive(Clone, Default)]
pub struct PartialServerInfo {
    info: ServerInfo,
    received: u64,
}

pub enum MergeError {
    DifferingTokens,
    DifferingVersions,
    NotV664,
    OverlappingPlayers,
}

impl PartialServerInfo {
    // TODO: What to do when the infos don't match?
    // Currently the other info is just ignored.
    pub fn merge(&mut self, other: PartialServerInfo) -> Result<(),MergeError> {
        if self.info.token != other.info.token {
            return Err(MergeError::DifferingTokens);
        }
        if self.info.info_version != other.info.info_version {
            return Err(MergeError::DifferingVersions);
        }
        if self.info.info_version != ServerInfoVersion::V664 {
            return Err(MergeError::NotV664);
        }
        if self.received & other.received == other.received {
            // We already have that server info.
            // TODO: What to do if it doesn't match?
            return Ok(());
        }
        if self.received & other.received != 0 {
            return Err(MergeError::OverlappingPlayers);
        }
        let mut iter = other.info.clients.into_iter();
        self.info.clients.extend(&mut iter);

        // Shouldn't happen because we keep a bitmask of players we already have.
        assert!(!iter.next().is_some(), "too many players");

        Ok(())
    }
    pub fn get_info(&self) -> Option<&ServerInfo> {
        if self.info.clients.len().to_i32().unwrap() != self.info.num_clients {
            return None;
        }
        Some(&self.info)
    }
}

fn debug_parse_fail(help: &str) -> Option<PartialServerInfo> {
    fn debug_parse_fail_impl(help: &str) {
        println!("server info parsing failed at {}", help);
        let _ = help;
    }
    debug_parse_fail_impl(help);
    None
}

fn parse_server_info<RI,RS>(
    unpacker: &mut Unpacker,
    read_int: RI,
    read_str: RS,
    version: ServerInfoVersion,
) -> Option<PartialServerInfo>
    where RI: FnMut(&mut Unpacker) -> Option<i32>,
          RS: for<'a> FnMut(&mut Unpacker<'a>) -> Option<&'a ZBytes>,
{
    use self::debug_parse_fail as fail;

    let mut read_int = read_int;
    let mut read_str = read_str;
    let mut result: PartialServerInfo = Default::default();

    macro_rules! int { ($cause:expr) => {
        unwrap_or_return!(read_int(unpacker), fail($cause));
    } }

    macro_rules! str { ($cause:expr) => {
        unwrap_or_return!(read_str(unpacker), fail($cause))
            .as_bytes_without_nul().iter().cloned().collect();
    } }

    {
        let i = &mut result.info;
        i.info_version = version;

        i.token = int!("token");
        i.version = str!("version");
        i.name = str!("name");
        if version.has_hostname() {
            i.hostname = Some(str!("hostname"));
        } else {
            i.hostname = None;
        }
        i.map         = str!("map");
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

        let offset;
        if version.has_offset() {
            offset = int!("offset");
        } else {
            offset = 0;
        }

        // Error handling copied from Teeworlds' source.
        if i.num_clients < 0 || i.num_clients > version.max_clients().to_i32().unwrap()
            || i.max_clients < 0 || i.max_clients > version.max_clients().to_i32().unwrap()
            || i.num_players < 0 || i.num_players > i.num_clients
            || i.max_players < 0 || i.max_players > i.max_clients
        {
            return fail("count sanity check");
        }

        let offset = unwrap_or_return!(offset.to_u32(), fail("offset sanity check"));

        let upper_limit = match version {
            ServerInfoVersion::V664 => MAX_CLIENTS.to_u32().unwrap(),
            _ => i.num_clients.to_u32().unwrap(),
        };

        for j in offset..upper_limit {
            let name = str!("client_name");
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
            if i.clients.push(ClientInfo {
                name: name,
                clan: clan,
                country: country,
                score: score,
                is_player: is_player,
            }).is_some() {
                return fail("too many clients");
            }
            result.received |= 1 << j;
        }
    }
    Some(result)
}

fn info_read_int_v5(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_string()
        .and_then(|x| str::from_utf8(x.as_bytes_without_nul()).ok())
        .and_then(|x| x.parse().ok())
}

fn info_read_int_v7(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_int()
}

fn info_read_str<'a>(unpacker: &mut Unpacker<'a>) -> Option<&'a ZBytes> {
    unpacker.read_string()
}

impl<'a> Info5Response<'a> {
    pub fn parse(self) -> Option<ServerInfo> {
        let Info5Response(slice) = self;
        let mut unpacker = Unpacker::from_slice(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ServerInfoVersion::V5,
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
          RS: for<'b> FnMut(&mut Unpacker<'b>) -> Option<&'b ZBytes>,
    {
        let Info6Response(slice) = self;
        let mut unpacker = Unpacker::from_slice(slice);
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
        let mut unpacker = Unpacker::from_slice(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ServerInfoVersion::V664,
        )
    }
}

#[derive(Copy, Clone)] pub struct Info5Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct Info6Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct Info664Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)] pub struct CountResponse(pub u16);
#[derive(Copy, Clone)] pub struct List5Response<'a>(pub &'a [Addr5Packed]);
#[derive(Copy, Clone)] pub struct List6Response<'a>(pub &'a [Addr6Packed]);

#[derive(Copy, Clone)]
pub enum Response<'a> {
    List5(List5Response<'a>),
    List6(List6Response<'a>),
    Count(CountResponse),
    Info5(Info5Response<'a>),
    Info6(Info6Response<'a>),
    Info664(Info664Response<'a>),
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
    let (header, data) = data.split_at(HEADER_LEN);
    let header: &[u8; HEADER_LEN] = unsafe { &*(header.as_ptr() as *const [u8; HEADER_LEN]) };
    match *header {
        LIST_5 => Some(Response::List5(List5Response(parse_list5(data)))),
        LIST_6 => Some(Response::List6(List6Response(parse_list6(data)))),
        INFO_5 => Some(Response::Info5(Info5Response(data))),
        INFO_6 => Some(Response::Info6(Info6Response(data))),
        INFO_6_64 => Some(Response::Info664(Info664Response(data))),
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
    use super::Info6Response;
    use super::ClientInfo;
    use super::ServerInfo;
    use super::ServerInfoVersion;

    #[test]
    fn parse_info_v7() {
        let info_raw = b"\x01two\0three\0four\0five\0six\0\x07\x08\x01\x02\x02\x03eleven\0twelve\0\x40\x0d\x0efifteen\0sixteen\0\x11\x12\x13";
        let info = ServerInfo {
            info_version: ServerInfoVersion::V7,
            token: 1,
            version: "two".as_bytes().iter().cloned().collect(),
            name: "three".as_bytes().iter().cloned().collect(),
            hostname: Some("four".as_bytes().iter().cloned().collect()),
            map: "five".as_bytes().iter().cloned().collect(),
            game_type: "six".as_bytes().iter().cloned().collect(),
            flags: 7,
            progression: None,
            skill_level: Some(8),
            num_players: 1,
            max_players: 2,
            num_clients: 2,
            max_clients: 3,
            clients: [
                ClientInfo {
                    name: "eleven".as_bytes().iter().cloned().collect(),
                    clan: "twelve".as_bytes().iter().cloned().collect(),
                    country: -1,
                    score: 13,
                    is_player: 14,
                },
                ClientInfo {
                    name: "fifteen".as_bytes().iter().cloned().collect(),
                    clan: "sixteen".as_bytes().iter().cloned().collect(),
                    country: 17,
                    score: 18,
                    is_player: 19,
                },
            ].iter().cloned().collect(),
        };

        println!("{:?}", Info6Response(info_raw).parse().unwrap());
        println!("{:?}", info);

        assert_eq!(Info6Response(info_raw).parse(), Some(info));
    }
}
