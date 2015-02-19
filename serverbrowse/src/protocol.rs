use common;
use common::num::BeU16;
use common::num::LeU16;

use std::cmp;
use std::default::Default;
use std::fmt;
use std::hash;
use std::mem;
use std::num::ToPrimitive;
use std::old_io::net::ip::IpAddr;
use std::old_io::net::ip::Ipv4Addr;
use std::old_io::net::ip::Ipv6Addr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::slice;
use std::str;

const MAX_CLIENTS:      u32 = 64;
const MAX_CLIENTS_5:    u32 = 16;
const MAX_CLIENTS_6_64: u32 = 64;

pub const MASTERSERVER_PORT: u16 = 8300;

/// Non-zero byte.
#[unstable = "definition might move into a different module/crate"]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NzU8(u8);

#[unstable]
impl NzU8 {
    #[unstable]
    pub fn from_u8(v: u8) -> NzU8 {
        assert!(v != 0);
        NzU8(v)
    }
    #[unstable]
    pub fn to_u8(self) -> u8 {
        let NzU8(v) = self;
        v
    }
}

impl fmt::Debug for NzU8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let NzU8(v) = *self;
        v.fmt(f)
    }
}

impl fmt::Display for NzU8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let NzU8(v) = *self;
        v.fmt(f)
    }
}

pub trait NzU8SliceExt {
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> Option<&Self>;
}

impl NzU8SliceExt for [NzU8] {
    fn as_bytes(&self) -> &[u8] {
        unsafe { mem::transmute(self) }
    }
    fn from_bytes(bytes: &[u8]) -> Option<&[NzU8]> {
        for &c in bytes.iter() {
            if c == 0 {
                return None;
            }
        }
        Some(unsafe { mem::transmute(bytes) })
    }
}

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
    pub fn as_nzbytes(&self) -> &[NzU8] {
        let len = self.as_bytes().len();
        unsafe { mem::transmute(&self.as_bytes()[..len - 1]) }
    }
    pub fn as_nzbytes_mut(&mut self) -> &mut [NzU8] {
        let len = self.as_bytes().len();
        unsafe { mem::transmute(&mut self.as_bytes_mut()[..len - 1]) }
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
    let slice = iter.as_slice();
    // `by_ref` is needed as the iterator is silently copied otherwise.
    for (i, &c) in iter.by_ref().enumerate() {
        if c == 0 {
            return Some(unsafe { mem::transmute(&slice[..i + 1]) });
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

const S: usize = 64;
#[derive(Copy)]
pub struct PString64 {
    len: usize,
    contents: [NzU8; S],
}

impl PString64 {
    pub fn new() -> PString64 {
        PString64 {
            len: 0,
            contents: unsafe { mem::uninitialized() },
        }
    }
    pub fn from_slice(slice: &[NzU8]) -> PString64 {
        assert!(slice.len() <= S);
        PString64::from_slice_trunc(slice)
    }
    pub fn from_str(slice: &str) -> PString64 {
        PString64::from_slice(NzU8SliceExt::from_bytes(slice.as_bytes()).unwrap())
    }
    pub fn from_slice_trunc(slice: &[NzU8]) -> PString64 {
        let mut result = PString64::new();
        for (src, dest) in slice.iter().zip(result.contents.as_mut_slice().iter_mut()) {
            *dest = *src;
        }
        result.len = slice.len();
        result
    }
    pub fn as_slice(&self) -> &[NzU8] {
        &self.contents.as_slice()[..self.len]
    }
    pub fn as_mut_slice(&mut self) -> &mut [NzU8] {
        &mut self.contents.as_mut_slice()[..self.len]
    }
}

impl Clone for PString64 {
    fn clone(&self) -> PString64 {
        PString64 {
            len: self.len,
            contents: self.contents,
        }
    }
}

impl Default for PString64 {
    fn default() -> PString64 {
        PString64::new()
    }
}

impl Deref for PString64 {
    type Target = [NzU8];
    fn deref(&self) -> &[NzU8] {
        self.as_slice()
    }
}

impl DerefMut for PString64 {
    fn deref_mut(&mut self) -> &mut [NzU8] {
        self.as_mut_slice()
    }
}

impl PartialEq for PString64 {
    fn eq(&self, other: &PString64) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for PString64 { }

impl<S:hash::Hasher+hash::Writer> hash::Hash<S> for PString64 {
    fn hash(&self, state: &mut S) {
        self.as_slice().hash(state);
    }
}

impl fmt::Debug for PString64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[allow(missing_copy_implementations)]
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct PlayerInfo {
    pub name: PString64,
    pub clan: PString64,
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}

impl fmt::Debug for PlayerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#""{:?}" "{:?}" {:?} {:?} {:?}"#,
            String::from_utf8_lossy(self.name.as_bytes()),
            String::from_utf8_lossy(self.clan.as_bytes()),
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

pub struct ServerInfo {
    pub info_version: ServerInfoVersion,
    pub token: i32,
    pub version: PString64,
    pub name: PString64,
    pub hostname: Option<PString64>,
    pub map: PString64,
    pub game_type: PString64,
    pub flags: i32,
    pub progression: Option<i32>,
    pub skill_level: Option<i32>,
    pub num_players: i32,
    pub max_players: i32,
    pub num_clients: i32,
    pub max_clients: i32,
    pub clients_array: [PlayerInfo; MAX_CLIENTS as usize],
}

impl ServerInfo {
    pub fn sort_clients(&mut self) {
        self.clients_mut().sort_by(|a, b| Ord::cmp(&*a.name, &*b.name));
    }
    pub fn real_num_clients(&self) -> u32 {
        let num_clients = self.num_clients.to_u32().unwrap_or(0);
        if num_clients <= MAX_CLIENTS {
            num_clients
        } else {
            MAX_CLIENTS
        }
    }
    pub fn clients(&self) -> &[PlayerInfo] {
        let len = self.real_num_clients();
        &self.clients_array[..len as usize]
    }
    pub fn clients_mut(&mut self) -> &mut [PlayerInfo] {
        let len = self.real_num_clients();
        &mut self.clients_array[..len as usize]
    }
}

impl fmt::Debug for ServerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"{:?} {:?} "{:?}" "{:?}" "{:?}" "{:?}" "{:?}" {:?} {:?} {:?} {:?}/{:?} {:?}/{:?}: {:?}"#,
            self.info_version,
            self.token,
            String::from_utf8_lossy(self.version.as_bytes()),
            String::from_utf8_lossy(self.name.as_bytes()),
            self.hostname.map(|x| String::from_utf8_lossy(x.as_bytes()).into_owned()),
            String::from_utf8_lossy(self.map.as_bytes()),
            String::from_utf8_lossy(self.game_type.as_bytes()),
            self.flags,
            self.progression,
            self.skill_level,
            self.num_players,
            self.max_players,
            self.num_clients,
            self.max_clients,
            self.clients(),
        )
    }
}

pub struct PartialServerInfo {
    info: ServerInfo,
    fill_array: [bool; MAX_CLIENTS as usize],
}

impl PartialServerInfo {
    fn fill(&self) -> &[bool] {
        &self.fill_array[..self.info.real_num_clients() as usize]
    }
    fn fill_mut(&mut self) -> &mut [bool] {
        &mut self.fill_array[..self.info.real_num_clients() as usize]
    }
    // TODO: What to do when the infos don't match?
    // Currently the other info is just ignored.
    pub fn merge(&mut self, other: PartialServerInfo) {
        if self.info.token != other.info.token
            || self.info.num_clients != other.info.num_players
        {
            return;
        }
        // TODO: Better loop.
        for (i, &f) in other.fill().iter().enumerate() {
            if f {
                // TODO: What if a client is received twice?
                self.info.clients_mut()[i] = other.info.clients()[i].clone();
                self.fill_mut()[i] = true;
            }
        }
    }
    pub fn get_info(&self) -> Option<&ServerInfo> {
        for &f in self.fill().iter() {
            if !f {
                return None;
            }
        }
        Some(&self.info)
    }
}

fn debug_parse_fail(help: &str) -> Option<PartialServerInfo> {
    fn debug_parse_fail_impl(help: &str) {
        //warn!("server info parsing failed at {}", help);
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
          RS: FnMut(&mut Unpacker) -> Option<PString64>,
{
    use self::debug_parse_fail as fail;

    let mut read_int = read_int;
    let mut read_str = read_str;
    let mut result: PartialServerInfo = Default::default();

    let (offset, end) = {
        let i = &mut result.info;
        i.info_version = version;

        i.token       = unwrap_or_return!(read_int(unpacker), fail("token"));
        i.version     = unwrap_or_return!(read_str(unpacker), fail("version"));
        i.name        = unwrap_or_return!(read_str(unpacker), fail("name"));
        if version.has_hostname() {
            i.hostname = Some(unwrap_or_return!(read_str(unpacker), fail("hostname")));
        } else {
            i.hostname = None;
        }
        i.map         = unwrap_or_return!(read_str(unpacker), fail("map"));
        i.game_type   = unwrap_or_return!(read_str(unpacker), fail("game_type"));
        i.flags       = unwrap_or_return!(read_int(unpacker), fail("flags"));
        if version.has_progression() {
            i.progression = Some(unwrap_or_return!(read_int(unpacker), fail("progression")));
        } else {
            i.progression = None;
        }
        if version.has_skill_level() {
            i.skill_level = Some(unwrap_or_return!(read_int(unpacker), fail("skill_level")));
        } else {
            i.skill_level = None;
        }
        i.num_players = unwrap_or_return!(read_int(unpacker), fail("num_players"));
        i.max_players = unwrap_or_return!(read_int(unpacker), fail("max_players"));
        if version.has_extended_player_info() {
            i.num_clients = unwrap_or_return!(read_int(unpacker), fail("num_clients"));
            i.max_clients = unwrap_or_return!(read_int(unpacker), fail("max_clients"));
        } else {
            i.num_clients = i.num_players;
            i.max_clients = i.max_players;
        }

        let offset;
        if version.has_offset() {
            offset = unwrap_or_return!(read_int(unpacker), fail("offset"));
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
        // 64p offset checking
        if offset != 0 && offset >= i.real_num_clients() {
            return fail("offset sanity check");
        }

        let end = cmp::min(offset.to_u32().unwrap() + version.clients_per_packet(), i.real_num_clients());

        for c in i.clients_mut()[offset as usize..end as usize].iter_mut() {
            c.name = unwrap_or_return!(read_str(unpacker), fail("name"));
            if version.has_extended_player_info() {
                c.clan    = unwrap_or_return!(read_str(unpacker), fail("clan"));
                c.country = unwrap_or_return!(read_int(unpacker), fail("country"));
            } else {
                c.clan    = PString64::new();
                c.country = -1;
            }
            c.score = unwrap_or_return!(read_int(unpacker), fail("score"));
            if version.has_extended_player_info() {
                c.is_player = unwrap_or_return!(read_int(unpacker), fail("is_player"));
            } else {
                c.is_player = 1;
            }
        }
        (offset as usize, end as usize)
    };
    for f in result.fill_mut()[offset..end].iter_mut() {
        *f = true;
    }

    Some(result)
}

fn info_read_int_v5(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_string()
        .and_then(|x| str::from_utf8(x.as_nzbytes().as_bytes()).ok())
        .and_then(|x| x.parse().ok())
}

fn info_read_int_v7(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_int()
}

fn info_read_str(unpacker: &mut Unpacker) -> Option<PString64> {
    unpacker.read_string()
        .map(|x| PString64::from_slice_trunc(x.as_nzbytes()))
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
          RS: FnMut(&mut Unpacker) -> Option<PString64>,
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
    unsafe { common::transmute_slice(data) }
}

fn parse_list6(data: &[u8]) -> &[Addr6Packed] {
    let remainder = data.len() % mem::size_of::<Addr6Packed>();
    if remainder != 0 {
        warn!("parsing overlong list5");
    }
    let data = &data[..data.len() - remainder];
    unsafe { common::transmute_slice(data) }
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

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Addr {
    pub ip_address: IpAddr,
    pub port: u16,
}

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ip_address {
            Ipv4Addr(..) => write!(f, "{}:{}", self.ip_address, self.port),
            Ipv6Addr(..) => write!(f, "[{}]:{}", self.ip_address, self.port),
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

impl fmt::Display for PString64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Clone for ServerInfo {
    fn clone(&self) -> ServerInfo {
        let c = &self.clients_array;
        ServerInfo {
            info_version: self.info_version.clone(),
            token:        self.token       .clone(),
            version:      self.version     .clone(),
            name:         self.name        .clone(),
            hostname:     self.hostname    .clone(),
            map:          self.map         .clone(),
            game_type:    self.game_type   .clone(),
            flags:        self.flags       .clone(),
            progression:  self.progression .clone(),
            skill_level:  self.skill_level .clone(),
            num_players:  self.num_players .clone(),
            max_players:  self.max_players .clone(),
            num_clients:  self.num_clients .clone(),
            max_clients:  self.max_clients .clone(),
            clients_array: [
                c[ 0].clone(), c[ 1].clone(), c[ 2].clone(), c[ 3].clone(), c[ 4].clone(),
                c[ 5].clone(), c[ 6].clone(), c[ 7].clone(), c[ 8].clone(), c[ 9].clone(),
                c[10].clone(), c[11].clone(), c[12].clone(), c[13].clone(), c[14].clone(),
                c[15].clone(), c[16].clone(), c[17].clone(), c[18].clone(), c[19].clone(),
                c[20].clone(), c[21].clone(), c[22].clone(), c[23].clone(), c[24].clone(),
                c[25].clone(), c[26].clone(), c[27].clone(), c[28].clone(), c[29].clone(),
                c[30].clone(), c[31].clone(), c[32].clone(), c[33].clone(), c[34].clone(),
                c[35].clone(), c[36].clone(), c[37].clone(), c[38].clone(), c[39].clone(),
                c[40].clone(), c[41].clone(), c[42].clone(), c[43].clone(), c[44].clone(),
                c[45].clone(), c[46].clone(), c[47].clone(), c[48].clone(), c[49].clone(),
                c[50].clone(), c[51].clone(), c[52].clone(), c[53].clone(), c[54].clone(),
                c[55].clone(), c[56].clone(), c[57].clone(), c[58].clone(), c[59].clone(),
                c[60].clone(), c[61].clone(), c[62].clone(), c[63].clone(),
            ]
        }
    }
}

impl Default for ServerInfo {
    fn default() -> ServerInfo {
        fn d<T:Default>() -> T { Default::default() }
        ServerInfo {
            info_version: ServerInfoVersion::V6,
            token:        d(),
            version:      d(),
            name:         d(),
            hostname:     d(),
            map:          d(),
            game_type:    d(),
            flags:        d(),
            progression:  d(),
            skill_level:  d(),
            num_players:  d(),
            max_players:  d(),
            num_clients:  d(),
            max_clients:  d(),
            clients_array: [
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
            ],
        }
    }
}

impl PartialEq for ServerInfo {
    fn eq(&self, other: &ServerInfo) -> bool {
        true
        && self.info_version == other.info_version
        && self.token        == other.token
        && self.version      == other.version
        && self.name         == other.name
        && self.hostname     == other.hostname
        && self.map          == other.map
        && self.game_type    == other.game_type
        && self.flags        == other.flags
        && self.progression  == other.progression
        && self.skill_level  == other.skill_level
        && self.num_players  == other.num_players
        && self.max_players  == other.max_players
        && self.num_clients  == other.num_clients
        && self.max_clients  == other.max_clients
        && self.clients()    == other.clients()
    }
}

impl Eq for ServerInfo { }

impl<S:hash::Hasher+hash::Writer> hash::Hash<S> for ServerInfo {
    fn hash(&self, state: &mut S) {
        self.info_version.hash(state);
        self.token       .hash(state);
        self.version     .hash(state);
        self.name        .hash(state);
        self.hostname    .hash(state);
        self.map         .hash(state);
        self.game_type   .hash(state);
        self.flags       .hash(state);
        self.progression .hash(state);
        self.skill_level .hash(state);
        self.num_players .hash(state);
        self.max_players .hash(state);
        self.num_clients .hash(state);
        self.max_clients .hash(state);
        self.clients()   .hash(state);
    }
}

impl Clone for PartialServerInfo {
    fn clone(&self) -> PartialServerInfo {
        let f = &self.fill_array;
        PartialServerInfo {
            info: self.info.clone(),
            fill_array: [
                f[ 0].clone(), f[ 1].clone(), f[ 2].clone(), f[ 3].clone(), f[ 4].clone(),
                f[ 5].clone(), f[ 6].clone(), f[ 7].clone(), f[ 8].clone(), f[ 9].clone(),
                f[10].clone(), f[11].clone(), f[12].clone(), f[13].clone(), f[14].clone(),
                f[15].clone(), f[16].clone(), f[17].clone(), f[18].clone(), f[19].clone(),
                f[20].clone(), f[21].clone(), f[22].clone(), f[23].clone(), f[24].clone(),
                f[25].clone(), f[26].clone(), f[27].clone(), f[28].clone(), f[29].clone(),
                f[30].clone(), f[31].clone(), f[32].clone(), f[33].clone(), f[34].clone(),
                f[35].clone(), f[36].clone(), f[37].clone(), f[38].clone(), f[39].clone(),
                f[40].clone(), f[41].clone(), f[42].clone(), f[43].clone(), f[44].clone(),
                f[45].clone(), f[46].clone(), f[47].clone(), f[48].clone(), f[49].clone(),
                f[50].clone(), f[51].clone(), f[52].clone(), f[53].clone(), f[54].clone(),
                f[55].clone(), f[56].clone(), f[57].clone(), f[58].clone(), f[59].clone(),
                f[60].clone(), f[61].clone(), f[62].clone(), f[63].clone(),
            ]
        }
    }
}

impl Default for PartialServerInfo {
    fn default() -> PartialServerInfo {
        fn d<T:Default>() -> T { Default::default() }
        PartialServerInfo {
            info: d(),
            fill_array: [
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
                d(), d(), d(), d(), d(), d(), d(), d(),
            ],
        }
    }
}

#[test] fn check_alignment_addr5_packed() { assert_eq!(mem::min_align_of::<Addr5Packed>(), 1); }

impl Addr5Packed {
    pub fn unpack(self) -> Addr {
        let Addr5Packed { ip_address, port } = self;
        Addr {
            ip_address: Ipv4Addr(ip_address[0], ip_address[1], ip_address[2], ip_address[3]),
            port: port.to_u16(),
        }
    }
}

#[test] fn check_alignment_addr6_packed() { assert_eq!(mem::min_align_of::<Addr6Packed>(), 1); }

impl Addr6Packed {
    pub fn unpack(self) -> Addr {
        let Addr6Packed { ip_address, port } = self;
        let (maybe_ipv4_mapping, ipv4_address) = ip_address.split_at(IPV4_MAPPING.len());
        let new_address = if maybe_ipv4_mapping != IPV4_MAPPING {
            let ip_address: [BeU16; 8] = unsafe { mem::transmute(ip_address) };
            Ipv6Addr(
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
            Ipv4Addr(ipv4_address[0], ipv4_address[1], ipv4_address[2], ipv4_address[3])
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
    use super::PString64;
    use super::PlayerInfo;
    use super::ServerInfo;
    use super::ServerInfoVersion;

    use std::default::Default;

    #[test]
    fn parse_info_v7() {
        let info_raw = bytes!("\x01two\0three\0four\0five\0six\0\x07\x08\x01\x02\x02\x03eleven\0twelve\0\x40\x0d\x0efifteen\0sixteen\0\x11\x12\x13");
        let info = ServerInfo {
            info_version: ServerInfoVersion::V7,
            token: 1,
            version: PString64::from_str("two"),
            name: PString64::from_str("three"),
            hostname: Some(PString64::from_str("four")),
            map: PString64::from_str("five"),
            game_type: PString64::from_str("six"),
            flags: 7,
            skill_level: Some(8),
            num_players: 1,
            max_players: 2,
            num_clients: 2,
            max_clients: 3,
            clients_array: [
                PlayerInfo {
                    name: PString64::from_str("eleven"),
                    clan: PString64::from_str("twelve"),
                    country: -1,
                    score: 13,
                    is_player: 14,
                },
                PlayerInfo {
                    name: PString64::from_str("fifteen"),
                    clan: PString64::from_str("sixteen"),
                    country: 17,
                    score: 18,
                    is_player: 19,
                },
                Default::default(), Default::default(),
                Default::default(), Default::default(),
                Default::default(), Default::default(),
                Default::default(), Default::default(),
                Default::default(), Default::default(),
                Default::default(), Default::default(),
                Default::default(), Default::default(),
            ]
        };

        println!("");
        println!("{:?}", Info6Response(info_raw).parse().unwrap());
        println!("{:?}", info);

        assert_eq!(Info6Response(info_raw).parse().unwrap(), info);
    }
}
