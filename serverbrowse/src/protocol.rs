use common;
use common::num::BeU16;
use common::num::LeU16;

use std::default::Default;
use std::fmt;
use std::hash;
use std::io::net::ip::IpAddr;
use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::Ipv6Addr;
use std::mem;
use std::num::ToPrimitive;
use std::ops::Deref;
use std::ops::DerefMut;
use std::slice;
use std::str;

const MAX_CLIENTS:     uint = 64;
const MAX_CLIENTS_5:   uint = 16;
const MAX_CLIENTS_664: uint = 64;

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

impl fmt::Show for NzU8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let NzU8(v) = *self;
        v.fmt(f)
    }
}

impl fmt::String for NzU8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let NzU8(v) = *self;
        v.fmt(f)
    }
}

pub trait NzU8Slice {
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> Option<&Self>;
}

impl NzU8Slice for [NzU8] {
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
    pub fn slice_from(&self, begin: uint) -> &ZBytes {
        assert!(begin < self.as_bytes().len(), "cannot slice nul byte away");
        unsafe { ZBytes::from_bytes_unchecked(self.as_bytes().slice_from(begin)) }
    }
    pub fn slice_from_mut(&mut self, begin: uint) -> &mut ZBytes {
        assert!(begin < self.as_bytes().len(), "cannot slice nul byte away");
        unsafe { ZBytes::from_bytes_unchecked_mut(self.as_bytes_mut().slice_from_mut(begin)) }
    }
    pub fn as_nzbytes(&self) -> &[NzU8] {
        let len = self.as_bytes().len();
        unsafe { mem::transmute(self.as_bytes().slice_to(len - 1)) }
    }
    pub fn as_nzbytes_mut(&mut self) -> &mut [NzU8] {
        let len = self.as_bytes().len();
        unsafe { mem::transmute(self.as_bytes_mut().slice_to_mut(len - 1)) }
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

    result |= (src & 0x3f) as i32; // 0x3f == 0b0011_1111

    for i in range(0u32, 4) {
        if src & 0x80 == 0 { // 0x80 == 0b1000_0000
            break;
        }
        src = *unwrap_or_return!(iter.next(), None);
        result |= ((src & 0x7f) as i32) << (6 + 7 * i as uint); // 0x7f == 0b0111_1111
    }

    result ^= -sign;

    Some(result)
}

pub fn read_string<'a>(iter: &mut slice::Iter<'a,u8>) -> Option<&'a ZBytes> {
    let slice = iter.as_slice();
    // `by_ref` is needed as the iterator is silently copied otherwise.
    for (i, &c) in iter.by_ref().enumerate() {
        if c == 0 {
            return Some(unsafe { mem::transmute(slice.slice_to(i + 1)) });
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
const HEADER_LEN: uint = 14;
pub type Header = [u8; HEADER_LEN];
pub const REQUEST_LIST_5:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'r' as u8, 'e' as u8, 'q' as u8, 't' as u8]; // "reqt"
pub const REQUEST_LIST_6:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'r' as u8, 'e' as u8, 'q' as u8, '2' as u8]; // "req2"
pub const LIST_5:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'l' as u8, 'i' as u8, 's' as u8, 't' as u8]; // "list"
pub const LIST_6:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'l' as u8, 'i' as u8, 's' as u8, '2' as u8]; // "lis2"
pub const REQUEST_COUNT:     Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'c' as u8, 'o' as u8, 'u' as u8, '2' as u8]; // "cou2"
pub const COUNT:             Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 's' as u8, 'i' as u8, 'z' as u8, '2' as u8]; // "siz2"
pub const REQUEST_INFO_5:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'g' as u8, 'i' as u8, 'e' as u8, '2' as u8]; // "gie2"
pub const REQUEST_INFO_6:    Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'g' as u8, 'i' as u8, 'e' as u8, '3' as u8]; // "gie3"
pub const REQUEST_INFO_6_64: Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'f' as u8, 's' as u8, 't' as u8, 'd' as u8]; // "fstd"
pub const INFO_5:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'i' as u8, 'n' as u8, 'f' as u8, '2' as u8]; // "inf2"
pub const INFO_6:            Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'i' as u8, 'n' as u8, 'f' as u8, '3' as u8]; // "inf3"
pub const INFO_6_64:         Header = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 'd' as u8, 't' as u8, 's' as u8, 'f' as u8]; // "dtsf"

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

const S: uint = 64;
#[derive(Copy)]
pub struct PString64 {
    len: uint,
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
        PString64::from_slice(NzU8Slice::from_bytes(slice.as_bytes()).unwrap())
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
        self.contents.as_slice().slice_to(self.len)
    }
    pub fn as_mut_slice(&mut self) -> &mut [NzU8] {
        self.contents.as_mut_slice().slice_to_mut(self.len)
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

impl fmt::Show for PString64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[derive(Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct PlayerInfo {
    pub name: PString64,
    pub clan: PString64,
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}

impl fmt::Show for PlayerInfo {
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

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd, Show)]
pub enum ServerInfoVersion {
    V5,
    V6,
    V664,
    V7,
}

impl ServerInfoVersion {
    pub fn max_clients(self) -> uint {
        match self {
            ServerInfoVersion::V5   => MAX_CLIENTS_5,
            ServerInfoVersion::V6   => MAX_CLIENTS_5,
            ServerInfoVersion::V664 => MAX_CLIENTS_664,
            ServerInfoVersion::V7   => MAX_CLIENTS_5,
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

#[derive(Copy)]
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
    pub clients_array: [PlayerInfo; MAX_CLIENTS],
}

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
struct ServerInfoRaw {
    pub offset: Option<i32>,
    pub rest: ServerInfo,
}

impl ServerInfo {
    pub fn sort_clients(&mut self) {
        self.clients_mut().sort_by(|a, b| Ord::cmp(&*a.name, &*b.name));
    }
    pub fn real_num_clients(&self) -> uint {
        let num_clients = self.num_clients.to_uint().unwrap_or(0);
        if num_clients <= MAX_CLIENTS {
            num_clients
        } else {
            MAX_CLIENTS
        }
    }
    pub fn clients(&self) -> &[PlayerInfo] {
        let len = self.real_num_clients();
        self.clients_array.slice_to(len)
    }
    pub fn clients_mut(&mut self) -> &mut [PlayerInfo] {
        let len = self.real_num_clients();
        self.clients_array.slice_to_mut(len)
    }
}

impl fmt::Show for ServerInfo {
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

impl Clone for ServerInfo { fn clone(&self) -> ServerInfo { *self } }

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

impl Default for ServerInfo {
    fn default() -> ServerInfo {
        ServerInfo {
            info_version: ServerInfoVersion::V6,
            token: Default::default(),
            version: Default::default(),
            name: Default::default(),
            hostname: Default::default(),
            map: Default::default(),
            game_type: Default::default(),
            flags: Default::default(),
            progression: Default::default(),
            skill_level: Default::default(),
            num_players: Default::default(),
            max_players: Default::default(),
            num_clients: Default::default(),
            max_clients: Default::default(),
            clients_array: [Default::default(); MAX_CLIENTS],
        }
    }
}

fn parse_server_info<RI,RS>(
    unpacker: &mut Unpacker,
    read_int: RI,
    read_str: RS,
    version: ServerInfoVersion,
) -> Option<ServerInfoRaw>
    where RI: FnMut(&mut Unpacker) -> Option<i32>,
          RS: FnMut(&mut Unpacker) -> Option<PString64>,
{
    let mut read_int = read_int;
    let mut read_str = read_str;
    let mut result: ServerInfoRaw = Default::default();

    {
        let i = &mut result.rest;
        i.info_version = version;

        i.token       = unwrap_or_return!(read_int(unpacker), None);
        i.version     = unwrap_or_return!(read_str(unpacker), None);
        i.name        = unwrap_or_return!(read_str(unpacker), None);
        if version.has_hostname() {
            i.hostname = Some(unwrap_or_return!(read_str(unpacker), None));
        } else {
            i.hostname = None;
        }
        i.map         = unwrap_or_return!(read_str(unpacker), None);
        i.game_type   = unwrap_or_return!(read_str(unpacker), None);
        i.flags       = unwrap_or_return!(read_int(unpacker), None);
        if version.has_progression() {
            i.progression = Some(unwrap_or_return!(read_int(unpacker), None));
        } else {
            i.progression = None;
        }
        if version.has_skill_level() {
            i.skill_level = Some(unwrap_or_return!(read_int(unpacker), None));
        } else {
            i.skill_level = None;
        }
        i.num_players = unwrap_or_return!(read_int(unpacker), None);
        i.max_players = unwrap_or_return!(read_int(unpacker), None);
        if version.has_extended_player_info() {
            i.num_clients = unwrap_or_return!(read_int(unpacker), None);
            i.max_clients = unwrap_or_return!(read_int(unpacker), None);
        } else {
            i.num_clients = i.num_players;
            i.max_clients = i.max_players;
        }

        if version.has_offset() {
            result.offset = Some(unwrap_or_return!(read_int(unpacker), None));
        } else {
            result.offset = None;
        }

        // Error handling copied from Teeworlds' source.
        if i.num_clients < 0 || i.num_clients > version.max_clients().to_i32().unwrap()
            || i.max_clients < 0 || i.max_clients > version.max_clients().to_i32().unwrap()
            || i.num_players < 0 || i.num_players > i.num_clients
            || i.max_players < 0 || i.max_players > i.max_clients
        {
            return None;
        }

        for c in i.clients_mut().iter_mut() {
            c.name = unwrap_or_return!(read_str(unpacker), None);
            if version.has_extended_player_info() {
                c.clan    = unwrap_or_return!(read_str(unpacker), None);
                c.country = unwrap_or_return!(read_int(unpacker), None);
            } else {
                c.clan    = PString64::new();
                c.country = -1;
            }
            c.score = unwrap_or_return!(read_int(unpacker), None);
            if version.has_extended_player_info() {
                c.is_player = unwrap_or_return!(read_int(unpacker), None);
            } else {
                c.is_player = 1;
            }
        }
    }

    Some(result)
}

fn info_read_int_v5(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_string()
        .and_then(|x| str::from_utf8(x.as_nzbytes().as_bytes()).ok())
        .and_then(|x| x.parse())
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
        ).map(|mut raw| { raw.rest.sort_clients(); raw.rest })
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
            .map(|mut raw| { raw.rest.sort_clients(); raw.rest })
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
    let data = data.slice_to(data.len() - remainder);
    unsafe { common::transmute_slice(data) }
}

fn parse_list6(data: &[u8]) -> &[Addr6Packed] {
    let remainder = data.len() % mem::size_of::<Addr6Packed>();
    if remainder != 0 {
        warn!("parsing overlong list5");
    }
    let data = data.slice_to(data.len() - remainder);
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

impl fmt::Show for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ip_address {
            Ipv4Addr(..) => write!(f, "{:?}:{:?}", self.ip_address, self.port),
            Ipv6Addr(..) => write!(f, "[{:?}]:{:?}", self.ip_address, self.port),
        }
    }
}

impl fmt::String for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ip_address {
            Ipv4Addr(..) => write!(f, "{:?}:{:?}", self.ip_address, self.port),
            Ipv6Addr(..) => write!(f, "[{:?}]:{:?}", self.ip_address, self.port),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Addr5Packed {
    ip_address: [u8; 4],
    port: LeU16,
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

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Addr6Packed {
    ip_address: [u8; 16],
    port: BeU16,
}

#[test] fn check_alignment_addr6_packed() { assert_eq!(mem::min_align_of::<Addr6Packed>(), 1); }

impl Addr6Packed {
    pub fn unpack(self) -> Addr {
        let Addr6Packed { ip_address, port } = self;
        let compare_with = ip_address.slice_to(IPV4_MAPPING.len());
        let new_address = if compare_with != IPV4_MAPPING {
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
            let ip_address = ip_address.slice_from(IPV4_MAPPING.len());
            Ipv4Addr(ip_address[0], ip_address[1], ip_address[2], ip_address[3])
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
