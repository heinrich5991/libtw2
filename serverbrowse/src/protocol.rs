use arrayvec::ArrayString;
use libtw2_common::num::Cast;
use libtw2_common::slice;
use libtw2_common::str::truncated_arraystring;
use libtw2_common::unwrap_or_return;
use libtw2_packer::Unpacker;
use std::default::Default;
use std::fmt;
use std::mem;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str;
use warn::Ignore;
use zerocopy::byteorder::big_endian;
use zerocopy::byteorder::little_endian;
use zerocopy::FromZeroes;
use zerocopy_derive::AsBytes;
use zerocopy_derive::FromBytes;
use zerocopy_derive::FromZeroes;

const PLAYER_MAX_NAME_LENGTH: usize = 16 - 1;
const PLAYER_MAX_CLAN_LENGTH: usize = 12 - 1;
const MAX_CLIENTS_5: u32 = 16;
const MAX_CLIENTS_6_64: u32 = 64;
const MAX_CLIENTS_7: u32 = 64;

pub const MASTERSERVER_PORT: u16 = 8300;
pub const MASTERSERVER_7_PORT: u16 = 8283;

const HEADER_LEN: usize = 14;
pub type Header = &'static [u8; HEADER_LEN];
pub const REQUEST_LIST_5: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffreqt";
pub const REQUEST_LIST_6: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffreq2";
pub const LIST_5: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfflist";
pub const LIST_6: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfflis2";
pub const REQUEST_COUNT: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffcou2";
pub const COUNT: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffsiz2";
pub const REQUEST_INFO_5: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffgie2";
pub const REQUEST_INFO_6: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffgie3";
pub const REQUEST_INFO_6_64: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfffstd";
pub const REQUEST_INFO_6_EX: Header = b"xe\0\0\0\0\xff\xff\xff\xffgie3";
pub const INFO_5: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffinf2";
pub const INFO_6: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffinf3";
pub const INFO_6_DDPER: Header = b"dp\0\0\0\0\xff\xff\xff\xffinf3";
pub const INFO_6_64: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffdtsf";
pub const INFO_6_EX: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffiext";
pub const INFO_6_EX_MORE: Header = b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffiex+";

pub const TOKEN_7: &'static [u8; 8] = b"\x04\0\0\xff\xff\xff\xff\x05";
pub const REQUEST_LIST_7: &'static [u8; 17] =
    b"\x21\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffreq2";
pub const LIST_7: &'static [u8; 17] = b"\x21\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfflis2";
pub const REQUEST_COUNT_7: &'static [u8; 17] =
    b"\x21\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffcou2";
pub const COUNT_7: &'static [u8; 17] = b"\x21\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffsiz2";
pub const REQUEST_INFO_7: &'static [u8; 17] =
    b"\x21\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffgie3";
pub const INFO_7: &'static [u8; 17] = b"\x21\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xffinf3";

pub const PACKETFLAG_CONNLESS: u8 = 1 << 6;
pub const SERVERINFO_FLAG_PASSWORDED: i32 = 1 << 0;
pub const SERVERINFO_FLAG_TIMESCORE: i32 = 1 << 1;

pub const IPV4_MAPPING: [u8; 12] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
];

// dont-send-http-servers@mastersrv.ddnet.org
// e02cb630-b680-38f6-81a6-da096e9696d1
pub const NO_BACKCOMPAT: &'static [u8; 16] = &[
    0xe0, 0x2c, 0xb6, 0x30, 0xb6, 0x80, 0x38, 0xf6, 0x81, 0xa6, 0xda, 0x09, 0x6e, 0x96, 0x96, 0xd1,
];

pub fn request_list_5() -> [u8; 14] {
    *REQUEST_LIST_5
}
pub fn request_list_6() -> [u8; 14] {
    *REQUEST_LIST_6
}
pub fn request_list_7(own_token: Token7, their_token: Token7) -> [u8; 17] {
    let mut request = [0; 17];
    request.copy_from_slice(REQUEST_LIST_7);
    request[1..5].copy_from_slice(&their_token.0);
    request[5..9].copy_from_slice(&own_token.0);
    request
}
pub fn request_list_5_nobackcompat() -> [u8; 30] {
    let mut request = [0; 30];
    request[..14].copy_from_slice(REQUEST_LIST_5);
    request[14..].copy_from_slice(NO_BACKCOMPAT);
    request
}
pub fn request_list_6_nobackcompat() -> [u8; 30] {
    let mut request = [0; 30];
    request[..14].copy_from_slice(REQUEST_LIST_6);
    request[14..].copy_from_slice(NO_BACKCOMPAT);
    request
}
pub fn request_list_7_nobackcompat(own_token: Token7, their_token: Token7) -> [u8; 33] {
    let mut request = [0; 33];
    request[..17].copy_from_slice(&request_list_7(own_token, their_token));
    request[17..].copy_from_slice(NO_BACKCOMPAT);
    request
}

pub fn request_info_5(challenge: u8) -> [u8; 15] {
    request_info(REQUEST_INFO_5, challenge)
}
pub fn request_info_6(challenge: u8) -> [u8; 15] {
    request_info(REQUEST_INFO_6, challenge)
}
pub fn request_info_6_64(challenge: u8) -> [u8; 15] {
    request_info(REQUEST_INFO_6_64, challenge)
}
pub fn request_info_6_ex(challenge: u32) -> [u8; 15] {
    assert!(
        challenge & 0x00ff_ffff == challenge,
        "only the lower 24 bits of challenge are used"
    );
    let mut request = [0; HEADER_LEN + 1];
    request[..HEADER_LEN].copy_from_slice(REQUEST_INFO_6_EX);
    request[2] = ((challenge & 0x00ff_0000) >> 16) as u8;
    request[3] = ((challenge & 0x0000_ff00) >> 8) as u8;
    request[HEADER_LEN] = ((challenge & 0x0000_00ff) >> 0) as u8;
    request
}
pub fn request_token_7(own_token: Token7) -> [u8; 520] {
    let mut request = [0; 520];
    request[..8].copy_from_slice(TOKEN_7);
    request[3..7].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    request[8..12].copy_from_slice(&own_token.0);
    request
}
pub fn request_info_7(own_token: Token7, their_token: Token7, challenge: u8) -> [u8; 18] {
    assert!(
        challenge & 0x3f == challenge,
        "only the lower 6 bits of challenge can be used with this implementation"
    );
    let mut request = [0; 18];
    request[..17].copy_from_slice(REQUEST_INFO_7);
    request[1..5].copy_from_slice(&their_token.0);
    request[5..9].copy_from_slice(&own_token.0);
    request[17] = challenge;
    request
}

pub fn request_count() -> [u8; 14] {
    *REQUEST_COUNT
}
pub fn request_count_nobackcompat() -> [u8; 30] {
    let mut request = [0; 30];
    request[..14].copy_from_slice(REQUEST_COUNT);
    request[14..].copy_from_slice(NO_BACKCOMPAT);
    request
}
pub fn request_count_7(own_token: Token7, their_token: Token7) -> [u8; 17] {
    let mut request = [0; 17];
    request.copy_from_slice(REQUEST_COUNT_7);
    request[1..5].copy_from_slice(&their_token.0);
    request[5..9].copy_from_slice(&own_token.0);
    request
}
pub fn request_count_7_nobackcompat(own_token: Token7, their_token: Token7) -> [u8; 33] {
    let mut request = [0; 33];
    request[..17].copy_from_slice(&request_count_7(own_token, their_token));
    request[17..].copy_from_slice(NO_BACKCOMPAT);
    request
}

fn request_info(header: Header, challenge: u8) -> [u8; 15] {
    let mut request = [0; HEADER_LEN + 1];
    request[..HEADER_LEN].copy_from_slice(header);
    request[HEADER_LEN] = challenge;
    request
}

#[derive(AsBytes, Clone, Copy, FromBytes, FromZeroes)]
#[repr(transparent)]
pub struct Token7(pub [u8; 4]);

impl fmt::Debug for Token7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl fmt::Display for Token7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClientInfo {
    pub name: ArrayString<[u8; PLAYER_MAX_NAME_LENGTH]>,
    pub clan: ArrayString<[u8; PLAYER_MAX_CLAN_LENGTH]>,
    pub country: i32,
    pub score: i32,
    pub flags: i32,
}

impl fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?} {:?}",
            self.name, self.clan, self.country, self.score, self.flags,
        )
    }
}

pub const CLIENTINFO_FLAG_SPECTATOR: i32 = 1 << 0;
pub const CLIENTINFO_FLAG_BOT: i32 = 1 << 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ServerInfoVersion {
    V5,
    V6,
    V6Ddper,
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
            ServerInfoVersion::V5 => MAX_CLIENTS_5,
            ServerInfoVersion::V6 => MAX_CLIENTS_5,
            ServerInfoVersion::V6Ddper => MAX_CLIENTS_5,
            ServerInfoVersion::V664 => MAX_CLIENTS_6_64,
            ServerInfoVersion::V6Ex => return None,
            ServerInfoVersion::V7 => MAX_CLIENTS_7,
        })
    }
    pub fn clients_per_packet(self) -> Option<u32> {
        Some(match self {
            ServerInfoVersion::V5 => 16,
            ServerInfoVersion::V6 => 16,
            ServerInfoVersion::V6Ddper => 16,
            ServerInfoVersion::V664 => 24,
            ServerInfoVersion::V6Ex => return None,
            ServerInfoVersion::V7 => 16,
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
    pub fn has_full_client_flags(self) -> bool {
        self == ServerInfoVersion::V7
    }
}

impl Default for ServerInfoVersion {
    fn default() -> ServerInfoVersion {
        ServerInfoVersion::V5
    }
}

#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct ServerInfo {
    pub info_version: ServerInfoVersion,
    pub token: i32,
    pub version: ArrayString<[u8; 32]>,
    pub name: ArrayString<[u8; 64]>,
    pub hostname: Option<ArrayString<[u8; 64]>>,
    pub map: ArrayString<[u8; 32]>,
    pub map_crc: Option<u32>,
    pub map_size: Option<u32>,
    pub game_type: ArrayString<[u8; 32]>,
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
        write!(
            f,
            "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}/{:?} {:?}/{:?}: {:?}",
            self.info_version,
            self.token,
            self.version,
            self.name,
            self.hostname,
            self.map,
            self.game_type,
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
    pub fn merge(&mut self, mut other: PartialServerInfo) -> Result<(), MergeError> {
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
        if self.info.info_version == ServerInfoVersion::V6Ex && self.received & 1 == 0 {
            mem::swap(self, &mut other);
        }
        self.info.clients.extend(other.info.clients.into_iter());

        Ok(())
    }
    pub fn token(&self) -> i32 {
        self.info.token
    }
    pub fn get_info(&mut self) -> Option<&ServerInfo> {
        if self.info.clients.len().assert_i32() != self.info.num_clients {
            return None;
        }
        self.info.clients.sort();
        Some(&self.info)
    }
    pub fn take_info(&mut self) -> Option<ServerInfo> {
        if self.get_info().is_none() {
            return None;
        }
        self.received = !0;
        Some(mem::replace(&mut self.info, Default::default()))
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

fn parse_server_info<RI, RS>(
    unpacker: &mut Unpacker,
    read_int: RI,
    read_str: RS,
    received_version: ReceivedServerInfoVersion,
) -> Option<PartialServerInfo>
where
    RI: FnMut(&mut Unpacker) -> Option<i32>,
    RS: for<'a> FnMut(&mut Unpacker<'a>) -> Option<&'a str>,
{
    use self::debug_parse_fail as fail;

    let mut read_int = read_int;
    let mut read_str = read_str;
    let mut result = PartialServerInfo::new();

    macro_rules! int {
        ($cause:expr) => {
            unwrap_or_return!(read_int(unpacker), fail($cause))
        };
    }

    macro_rules! str {
        ($cause:expr) => {
            truncated_arraystring(unwrap_or_return!(read_str(unpacker), fail($cause)))
        };
    }

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
            i.map = str!("map");
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
            i.game_type = str!("game_type");
            i.flags = int!("flags");
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
            if i.num_clients < 0
                || i.num_clients > i.max_clients
                || i.max_clients < 0
                || version
                    .max_clients()
                    .map(|m| i.max_clients > m.assert_i32())
                    .unwrap_or(false)
                || i.num_players < 0
                || i.num_players > i.num_clients
                || i.max_players < 0
                || i.max_players > i.max_clients
            {
                return fail("count sanity check");
            }
            offset = unwrap_or_return!(raw_offset.try_u32(), fail("offset sanity check"));
        }
        if version.has_extra_info() {
            let _: ArrayString<[u8; 0]> = str!("extra_info");
        }

        if version == ServerInfoVersion::V6Ex {
            result.received |= 1 << packet_no;
        }

        for j in offset.. {
            let name = match read_str(unpacker) {
                Some(n) => truncated_arraystring(n),
                None => break,
            };
            let clan;
            let country;
            if version.has_extended_player_info() {
                clan = str!("client_clan");
                country = int!("client_country");
            } else {
                clan = Default::default();
                country = -1;
            }
            let score = int!("client_score");
            let flags;
            if version.has_extended_player_info() {
                if version.has_full_client_flags() {
                    flags = int!("client_flags");
                } else {
                    let is_spectator = int!("client_is_player") == 0;
                    flags = if is_spectator {
                        CLIENTINFO_FLAG_SPECTATOR
                    } else {
                        0
                    };
                }
            } else {
                flags = 0;
            }
            if version.has_extra_info() {
                let _: ArrayString<[u8; 0]> = str!("extra_info");
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
                flags: flags,
            });
        }
    }
    Some(result)
}

fn info_read_int_v5(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker
        .read_string()
        .ok()
        .and_then(|x| str::from_utf8(x).ok())
        .and_then(|x| x.parse().ok())
}

fn info_read_int_v7(unpacker: &mut Unpacker) -> Option<i32> {
    unpacker.read_int(&mut Ignore).ok()
}

fn info_read_str<'a>(unpacker: &mut Unpacker<'a>) -> Option<&'a str> {
    unpacker
        .read_string()
        .ok()
        .and_then(|s| str::from_utf8(s).ok())
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
        )
        .map(|mut raw| {
            raw.info.sort_clients();
            raw.info
        })
    }
}

impl<'a> Info6Response<'a> {
    pub fn parse(self) -> Option<ServerInfo> {
        let Info6Response(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ReceivedServerInfoVersion::Normal(ServerInfoVersion::V6),
        )
        .map(|mut raw| {
            raw.info.sort_clients();
            raw.info
        })
    }
}

impl<'a> Info6DdperResponse<'a> {
    pub fn parse(self) -> Option<ServerInfo> {
        let Info6DdperResponse(slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v5,
            info_read_str,
            ReceivedServerInfoVersion::Normal(ServerInfoVersion::V6Ddper),
        )
        .map(|mut raw| {
            raw.info.sort_clients();
            raw.info
        })
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

impl<'a> Info7Response<'a> {
    pub fn parse(self) -> Option<ServerInfo> {
        let Info7Response(_, _, slice) = self;
        let mut unpacker = Unpacker::new(slice);
        parse_server_info(
            &mut unpacker,
            info_read_int_v7,
            info_read_str,
            ReceivedServerInfoVersion::Normal(ServerInfoVersion::V7),
        )
        .map(|mut raw| {
            raw.info.sort_clients();
            raw.info
        })
    }
}

#[derive(Copy, Clone)]
pub struct Info5Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct Info6Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct Info6DdperResponse<'a>(pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct Info664Response<'a>(pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct Info6ExResponse<'a>(pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct Info6ExMoreResponse<'a>(pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct Info7Response<'a>(pub Token7, pub Token7, pub &'a [u8]);
#[derive(Copy, Clone)]
pub struct CountResponse(pub u16);
#[derive(Copy, Clone)]
pub struct Count7Response(pub Token7, pub Token7, pub u16);
#[derive(Copy, Clone)]
pub struct List5Response<'a>(pub &'a [Addr5Packed]);
#[derive(Copy, Clone)]
pub struct List6Response<'a>(pub &'a [Addr6Packed]);
#[derive(Copy, Clone)]
pub struct List7Response<'a>(pub Token7, pub Token7, pub &'a [Addr6Packed]);
#[derive(Copy, Clone)]
pub struct Token7Response(pub Token7, pub Token7);

#[derive(Copy, Clone)]
pub enum Response<'a> {
    List5(List5Response<'a>),
    List6(List6Response<'a>),
    List7(List7Response<'a>),
    Count(CountResponse),
    Count7(Count7Response),
    Info5(Info5Response<'a>),
    Info6(Info6Response<'a>),
    Info6Ddper(Info6DdperResponse<'a>),
    Info664(Info664Response<'a>),
    Info6Ex(Info6ExResponse<'a>),
    Info6ExMore(Info6ExMoreResponse<'a>),
    Info7(Info7Response<'a>),
    Token7(Token7Response),
}

fn parse_list5(data: &[u8]) -> &[Addr5Packed] {
    let remainder = data.len() % mem::size_of::<Addr5Packed>();
    if remainder != 0 {
        warn!("parsing overlong list5");
    }
    let data = &data[..data.len() - remainder];
    unsafe { slice::transmute(data) }
}

fn parse_list6(data: &[u8]) -> &[Addr6Packed] {
    let remainder = data.len() % mem::size_of::<Addr6Packed>();
    if remainder != 0 {
        warn!("parsing overlong list5");
    }
    let data = &data[..data.len() - remainder];
    unsafe { slice::transmute(data) }
}

fn parse_token7(data: &[u8]) -> Option<Token7> {
    if data.len() < 4 {
        return None;
    }
    Some(Token7([data[0], data[1], data[2], data[3]]))
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
    match data.first() {
        Some(0x04) => {
            if data.len() < TOKEN_7.len() {
                return None;
            }
            let mut header = [0; 8];
            let mut own_token = Token7::new_zeroed();
            let payload = &data[8..];
            header.copy_from_slice(&data[..8]);
            own_token.0.copy_from_slice(&header[3..7]);
            for b in &mut header[3..7] {
                *b = 0xff;
            }
            return match &header {
                TOKEN_7 => Some(Response::Token7(Token7Response(
                    own_token,
                    parse_token7(payload)?,
                ))),
                _ => None,
            };
        }
        Some(0x21) => {
            if data.len() < 17 {
                return None;
            }
            let mut header = [0; 17];
            let mut own_token = Token7::new_zeroed();
            let mut their_token = Token7::new_zeroed();
            let payload = &data[17..];
            header.copy_from_slice(&data[..17]);
            own_token.0.copy_from_slice(&header[1..5]);
            their_token.0.copy_from_slice(&header[5..9]);
            for b in &mut header[1..9] {
                *b = 0xff;
            }
            return match &header {
                LIST_7 => Some(Response::List7(List7Response(
                    own_token,
                    their_token,
                    parse_list6(payload),
                ))),
                INFO_7 => Some(Response::Info7(Info7Response(
                    own_token,
                    their_token,
                    payload,
                ))),
                COUNT_7 => parse_count(payload)
                    .map(|x| Response::Count7(Count7Response(own_token, their_token, x))),
                _ => None,
            };
        }
        _ => {}
    }
    if data.len() < HEADER_LEN {
        return None;
    }
    if data[0] & PACKETFLAG_CONNLESS == 0 {
        return None;
    }
    let (header, data) = data.split_at(HEADER_LEN);
    let mut header: [u8; HEADER_LEN] = *unsafe { &*(header.as_ptr() as *const [u8; HEADER_LEN]) };
    if header[..2] != *b"dp" || header[6..] != INFO_6_DDPER[6..] {
        for b in &mut header[..6] {
            *b = 0xff;
        }
    } else {
        for b in &mut header[2..6] {
            *b = 0;
        }
    }
    match &header {
        LIST_5 => Some(Response::List5(List5Response(parse_list5(data)))),
        LIST_6 => Some(Response::List6(List6Response(parse_list6(data)))),
        INFO_5 => Some(Response::Info5(Info5Response(data))),
        INFO_6 => Some(Response::Info6(Info6Response(data))),
        INFO_6_DDPER => Some(Response::Info6Ddper(Info6DdperResponse(data))),
        INFO_6_64 => Some(Response::Info664(Info664Response(data))),
        INFO_6_EX => Some(Response::Info6Ex(Info6ExResponse(data))),
        INFO_6_EX_MORE => Some(Response::Info6ExMore(Info6ExMoreResponse(data))),
        COUNT => parse_count(data).map(|x| Response::Count(CountResponse(x))),
        _ => None,
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Addr {
    pub ip_address: IpAddr,
    pub port: u16,
}

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.ip_address {
            IpAddr::V4(..) => write!(f, "{}:{}", self.ip_address, self.port),
            IpAddr::V6(..) => write!(f, "[{}]:{}", self.ip_address, self.port),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Addr5Packed {
    ip_address: [u8; 4],
    port: little_endian::U16,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Addr6Packed {
    ip_address: [u8; 16],
    port: big_endian::U16,
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[test]
fn check_alignment_addr5_packed() {
    assert_eq!(mem::align_of::<Addr5Packed>(), 1);
}

impl Addr5Packed {
    pub fn unpack(self) -> Addr {
        let Addr5Packed { ip_address, port } = self;
        Addr {
            ip_address: Ipv4Addr::new(ip_address[0], ip_address[1], ip_address[2], ip_address[3])
                .into(),
            port: port.get(),
        }
    }
}

#[test]
fn check_alignment_addr6_packed() {
    assert_eq!(mem::align_of::<Addr6Packed>(), 1);
}

impl Addr6Packed {
    pub fn unpack(self) -> Addr {
        let Addr6Packed { ip_address, port } = self;
        let (maybe_ipv4_mapping, ipv4_address) = ip_address.split_at(IPV4_MAPPING.len());
        let new_address = if maybe_ipv4_mapping != IPV4_MAPPING {
            let ip_address: [big_endian::U16; 8] = unsafe { mem::transmute(ip_address) };
            Ipv6Addr::new(
                ip_address[0].get(),
                ip_address[1].get(),
                ip_address[2].get(),
                ip_address[3].get(),
                ip_address[4].get(),
                ip_address[5].get(),
                ip_address[6].get(),
                ip_address[7].get(),
            )
            .into()
        } else {
            Ipv4Addr::new(
                ipv4_address[0],
                ipv4_address[1],
                ipv4_address[2],
                ipv4_address[3],
            )
            .into()
        };
        Addr {
            ip_address: new_address,
            port: port.get(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ClientInfo;
    use super::Info6ExMoreResponse;
    use super::Info6ExResponse;
    use super::Info6Response;
    use super::Info7Response;
    use super::ServerInfo;
    use super::ServerInfoVersion;
    use super::Token7;
    use super::CLIENTINFO_FLAG_SPECTATOR;
    use libtw2_common::str::truncated_arraystring as b;

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
                    flags: 0,
                },
                ClientInfo {
                    name: b("seven"),
                    clan: b("eight"),
                    country: -1,
                    score: 9,
                    flags: 0,
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
                ClientInfo {
                    name: b("player1"),
                    clan: b("clan1"),
                    country: 1,
                    score: 11,
                    flags: CLIENTINFO_FLAG_SPECTATOR,
                },
                ClientInfo {
                    name: b("player2"),
                    clan: b("clan2"),
                    country: 2,
                    score: 22,
                    flags: CLIENTINFO_FLAG_SPECTATOR,
                },
                ClientInfo {
                    name: b("player3"),
                    clan: b("clan3"),
                    country: 3,
                    score: 33,
                    flags: 0,
                },
                ClientInfo {
                    name: b("player4"),
                    clan: b("clan4"),
                    country: 4,
                    score: 44,
                    flags: CLIENTINFO_FLAG_SPECTATOR,
                },
                ClientInfo {
                    name: b("player5"),
                    clan: b("clan5"),
                    country: 5,
                    score: 55,
                    flags: CLIENTINFO_FLAG_SPECTATOR,
                },
                ClientInfo {
                    name: b("player6"),
                    clan: b("clan6"),
                    country: 6,
                    score: 66,
                    flags: CLIENTINFO_FLAG_SPECTATOR,
                },
                ClientInfo {
                    name: b("player7"),
                    clan: b("clan7"),
                    country: 7,
                    score: 77,
                    flags: 0,
                },
                ClientInfo {
                    name: b("player8"),
                    clan: b("clan8"),
                    country: 8,
                    score: 88,
                    flags: 0,
                },
                ClientInfo {
                    name: b("player9"),
                    clan: b("clan9"),
                    country: 9,
                    score: 99,
                    flags: CLIENTINFO_FLAG_SPECTATOR,
                },
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

    #[test]
    fn parse_info_v7() {
        let info_raw = b"\x01two\0three\0four\0five\0six\0\x07\x08\x01\x02\x02\x03thirteen\0fourteen\0\x0f\x10\x11eighteen\0nineteen\0\x14\x15\x16";
        let info = ServerInfo {
            info_version: ServerInfoVersion::V7,
            token: 1,
            version: b("two"),
            name: b("three"),
            hostname: Some(b("four")),
            map: b("five"),
            map_crc: None,
            map_size: None,
            game_type: b("six"),
            flags: 7,
            progression: None,
            skill_level: Some(8),
            num_players: 1,
            max_players: 2,
            num_clients: 2,
            max_clients: 3,
            clients: vec![
                ClientInfo {
                    name: b("eighteen"),
                    clan: b("nineteen"),
                    country: 20,
                    score: 21,
                    flags: 22,
                },
                ClientInfo {
                    name: b("thirteen"),
                    clan: b("fourteen"),
                    country: 15,
                    score: 16,
                    flags: 17,
                },
            ],
        };
        assert_eq!(
            Info7Response(Token7([0, 0, 0, 0]), Token7([0, 0, 0, 0]), info_raw).parse(),
            Some(info),
        );
    }
}
