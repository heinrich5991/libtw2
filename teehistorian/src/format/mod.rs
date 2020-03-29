use chrono::DateTime;
use chrono::FixedOffset;
use packer::UnexpectedEnd;
use packer::Unpacker;
use serde_json;
use std::borrow::Cow;
use uuid::Uuid;

pub use self::item::Item;

pub mod item;

pub const MAGIC_LEN: usize = 16;
pub const UUID: [u8; MAGIC_LEN] = [
    // "699db17b-8efb-34ff-b1d8-da6f60c15dd1"
    0x69, 0x9d, 0xb1, 0x7b, 0x8e, 0xfb, 0x34, 0xff,
    0xb1, 0xd8, 0xda, 0x6f, 0x60, 0xc1, 0x5d, 0xd1,
];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Version {
    V1,
    V2,
}

impl Version {
    fn has_ex(self) -> bool {
        self != Version::V1
    }
}

#[derive(Debug)]
pub struct Header<'a> {
    pub version: i32,
    pub game_uuid: Uuid,
    pub timestamp: DateTime<FixedOffset>,
    pub server_port: u16,
    pub map_name: Cow<'a, str>,
    pub map_size: u32,
    pub map_crc: u32,
}

#[derive(Debug)]
pub enum MaybeEnd<E> {
    Err(E),
    UnexpectedEnd,
}

impl<E> From<UnexpectedEnd> for MaybeEnd<E> {
    fn from(_: UnexpectedEnd) -> MaybeEnd<E> {
        MaybeEnd::UnexpectedEnd
    }
}

#[derive(Debug)]
pub enum HeaderError {
    WrongMagic,
    MalformedJson,
    MalformedHeader,
    MalformedVersion,
    MalformedGameUuid,
    MalformedStartTime,
    MalformedServerPort,
    MalformedMapSize,
    MalformedMapCrc,
}

impl From<WrongMagic> for HeaderError {
    fn from(_: WrongMagic) -> HeaderError {
        HeaderError::WrongMagic
    }
}

impl From<HeaderError> for MaybeEnd<HeaderError> {
    fn from(e: HeaderError) -> MaybeEnd<HeaderError> {
        MaybeEnd::Err(e)
    }
}

#[derive(Debug)]
pub struct WrongMagic;

impl From<WrongMagic> for MaybeEnd<WrongMagic> {
    fn from(e: WrongMagic) -> MaybeEnd<WrongMagic> {
        MaybeEnd::Err(e)
    }
}

pub fn read_magic(p: &mut Unpacker) -> Result<(), MaybeEnd<WrongMagic>> {
    let magic = p.read_raw(MAGIC_LEN)?;
    if magic != UUID {
        return Err(WrongMagic.into());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct JsonHeader<'a> {
    version: Cow<'a, str>,
    game_uuid: Cow<'a, str>,
    start_time: Cow<'a, str>,
    server_port: Cow<'a, str>,
    map_name: Cow<'a, str>,
    map_size: Cow<'a, str>,
    map_crc: Cow<'a, str>,
}

pub fn read_header<'a>(p: &mut Unpacker<'a>)
    -> Result<Header<'a>, MaybeEnd<HeaderError>>
{
    use self::HeaderError::*;
    let header_data = p.read_string()?;
    let json_header: JsonHeader = serde_json::from_slice(header_data)
        .map_err(|e| if e.is_data() { MalformedHeader } else { MalformedJson })?;
    let version = json_header.version.parse().map_err(|_| MalformedVersion)?;
    let header = Header {
        version: version,
        game_uuid: json_header.game_uuid.parse().map_err(|_| MalformedGameUuid)?,
        timestamp: (if version == 1 {
            DateTime::parse_from_str(&json_header.start_time, "%Y-%m-%d %H:%M:%S %z")
        } else {
            json_header.start_time.parse()
        }).map_err(|_| MalformedStartTime)?,
        server_port: json_header.server_port.parse().map_err(|_| MalformedServerPort)?,
        map_name: json_header.map_name,
        map_size: json_header.map_size.parse().map_err(|_| MalformedMapSize)?,
        map_crc: u32::from_str_radix(&json_header.map_crc, 16).map_err(|_| MalformedMapCrc)?,
    };
    Ok(header)
}

impl From<HeaderError> for Error {
    fn from(e: HeaderError) -> Error {
        Error::Header(e)
    }
}

impl From<item::Error> for Error {
    fn from(e: item::Error) -> Error {
        Error::Item(e)
    }
}

#[derive(Debug)]
pub enum Error {
    Header(HeaderError),
    Item(item::Error),
    UnknownVersion,
    TickOverflow,
    UnexpectedEnd,
    InvalidClientId,
    PlayerNewDuplicate,
    PlayerDiffWithoutNew,
    PlayerOldWithoutNew,
    InputNewDuplicate,
    InputDiffWithoutNew,
}

#[cfg(test)]
mod test {
    fn assert_uuid(uuid: [u8; 16], identifier: &str) {
        use uuid::Uuid;

        const UUID_TEEWORLDS: [u8; 16] = [
            // "e05ddaaa-c4e6-4cfb-b642-5d48e80c0029"
            0xe0, 0x5d, 0xda, 0xaa, 0xc4, 0xe6, 0x4c, 0xfb,
            0xb6, 0x42, 0x5d, 0x48, 0xe8, 0x0c, 0x00, 0x29,
        ];

        let ns = Uuid::from_bytes(&UUID_TEEWORLDS).unwrap();
        let ours = Uuid::from_bytes(&uuid).unwrap();
        let correct = Uuid::new_v3(&ns, identifier);
        assert_eq!(ours, correct);
    }

    #[test]
    fn correct_uuids() {
        use super::UUID;
        use super::item;
        assert_uuid(UUID, "teehistorian@ddnet.tw");
        assert_uuid(item::UUID_AUTH_INIT, "teehistorian-auth-init@ddnet.tw");
        assert_uuid(item::UUID_AUTH_LOGIN, "teehistorian-auth-login@ddnet.tw");
        assert_uuid(item::UUID_AUTH_LOGOUT, "teehistorian-auth-logout@ddnet.tw");
    }
}
