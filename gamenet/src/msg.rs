use buffer::CapacityError;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::with_packer;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct IntegerData<'a> {
    inner: &'a [u8],
}

impl<'a> IntegerData<'a> {
    fn from_bytes(bytes: &[u8]) -> IntegerData {
        IntegerData {
            inner: bytes,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        self.inner
    }
}

#[derive(Clone, Copy, Debug)]
enum SystemOrGame<S, G> {
    System(S),
    Game(G),
}

impl<S, G> SystemOrGame<S, G> {
    fn is_game(&self) -> bool {
        match *self {
            SystemOrGame::System(_) => false,
            SystemOrGame::Game(_) => true,
        }
    }
    fn is_system(&self) -> bool {
        !self.is_game()
    }
}

impl SystemOrGame<i32, i32> {
    fn decode_id(id: i32) -> SystemOrGame<i32, i32> {
        let sys = id & 1 != 0;
        let msg = id >> 1;
        if sys {
            SystemOrGame::System(msg)
        } else {
            SystemOrGame::Game(msg)
        }
    }
    fn internal_id(self) -> i32 {
        match self {
            SystemOrGame::System(msg) => msg,
            SystemOrGame::Game(msg) => msg,
        }
    }
    fn encode_id(self) -> i32 {
        let iid = self.internal_id() as u32;
        assert!((iid & (1 << 31)) == 0);
        let flag = if self.is_system() { 1 } else { 0 };
        ((iid << 1) | flag) as i32
    }
}

impl<'a> System<'a> {
    pub fn decode_complete(p: &mut Unpacker<'a>) -> Result<System<'a>, Error> {
        if let SystemOrGame::System(msg_id) = SystemOrGame::decode_id(try!(p.read_int())) {
            System::decode(msg_id, p)
        } else {
            Err(Error::new())
        }
    }
    pub fn encode_complete<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::System(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode(p)));
        Ok(p.written())
    }
}

pub mod system {
    use buffer::CapacityError;
    use bytes::PrettyBytes;
    use error::Error;
    use packer::Packer;
    use packer::Unpacker;
    use std::fmt;
    use super::IntegerData;

    pub const INFO: i32 = 1;
    pub const MAP_CHANGE: i32 = 2;
    pub const MAP_DATA: i32 = 3;
    pub const CON_READY: i32 = 4;
    pub const SNAP: i32 = 5;
    pub const SNAP_EMPTY: i32 = 6;
    pub const SNAP_SINGLE: i32 = 7;
    pub const INPUT_TIMING: i32 = 9;
    pub const RCON_AUTH_STATUS: i32 = 10;
    pub const RCON_LINE: i32 = 11;
    pub const READY: i32 = 14;
    pub const ENTER_GAME: i32 = 15;
    pub const INPUT: i32 = 16;
    pub const RCON_CMD: i32 = 17;
    pub const RCON_AUTH: i32 = 18;
    pub const REQUEST_MAP_DATA: i32 = 19;
    pub const PING: i32 = 20;
    pub const PING_REPLY: i32 = 21;
    pub const RCON_CMD_ADD: i32 = 25;
    pub const RCON_CMD_REMOVE: i32 = 26;

    #[derive(Clone, Copy)]
    pub struct Info<'a> {
        pub version: &'a [u8],
        pub password: Option<&'a [u8]>,
    }

    #[derive(Clone, Copy)]
    pub struct MapChange<'a> {
        pub name: &'a [u8],
        pub crc: i32,
        pub size: i32,
    }

    #[derive(Clone, Copy)]
    pub struct MapData<'a> {
        pub last: i32,
        pub crc: i32,
        pub chunk: i32,
        pub data: &'a [u8],
    }

    #[derive(Clone, Copy)]
    pub struct ConReady;

    #[derive(Clone, Copy)]
    pub struct Snap<'a> {
        pub tick: i32,
        pub delta_tick: i32,
        pub num_parts: i32,
        pub part: i32,
        pub crc: i32,
        pub data: &'a [u8],
    }

    #[derive(Clone, Copy)]
    pub struct SnapEmpty {
        pub tick: i32,
        pub delta_tick: i32,
    }

    #[derive(Clone, Copy)]
    pub struct SnapSingle<'a> {
        pub tick: i32,
        pub delta_tick: i32,
        pub crc: i32,
        pub data: &'a [u8],
    }

    #[derive(Clone, Copy)]
    pub struct InputTiming {
        pub input_pred_tick: i32,
        pub time_left: i32,
    }

    #[derive(Clone, Copy)]
    pub struct RconAuthStatus {
        pub auth_level: Option<i32>,
        pub receive_commands: Option<i32>,
    }

    #[derive(Clone, Copy)]
    pub struct RconLine<'a> {
        pub line: &'a [u8],
    }

    #[derive(Clone, Copy)]
    pub struct Ready;

    #[derive(Clone, Copy)]
    pub struct EnterGame;

    #[derive(Clone, Copy)]
    pub struct Input<'a> {
        pub ack_snapshot: i32,
        pub intended_tick: i32,
        pub input: IntegerData<'a>,
    }

    #[derive(Clone, Copy)]
    pub struct RconCmd<'a> {
        pub cmd: &'a [u8],
    }

    #[derive(Clone, Copy)]
    pub struct RconAuth<'a> {
        pub _unused: &'a [u8],
        pub password: &'a [u8],
        pub request_commands: Option<i32>,
    }

    #[derive(Clone, Copy)]
    pub struct RequestMapData {
        pub chunk: i32,
    }

    #[derive(Clone, Copy)]
    pub struct Ping;

    #[derive(Clone, Copy)]
    pub struct PingReply;

    #[derive(Clone, Copy)]
    pub struct RconCmdAdd<'a> {
        pub name: &'a [u8],
        pub help: &'a [u8],
        pub params: &'a [u8],
    }

    #[derive(Clone, Copy)]
    pub struct RconCmdRemove<'a> {
        pub name: &'a [u8],
    }

    impl<'a> fmt::Debug for Info<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Info")
                .field("version", &PrettyBytes::new(&self.version))
                .field("password", &self.password.as_ref().map(|password| PrettyBytes::new(password)))
                .finish()
        }
    }
    impl<'a> Info<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<Info<'a>, Error> {
            Ok(Info {
                version: try!(_p.read_string()),
                password: _p.read_string().ok(),
            })
        }
    }

    impl<'a> fmt::Debug for MapChange<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("MapChange")
                .field("name", &PrettyBytes::new(&self.name))
                .field("crc", &self.crc)
                .field("size", &self.size)
                .finish()
        }
    }
    impl<'a> MapChange<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<MapChange<'a>, Error> {
            Ok(MapChange {
                name: try!(_p.read_string()),
                crc: try!(_p.read_int()),
                size: try!(_p.read_int()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self.name));
            try!(_p.write_int(self.crc));
            try!(_p.write_int(self.size));
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for MapData<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("MapData")
                .field("last", &self.last)
                .field("crc", &self.crc)
                .field("chunk", &self.chunk)
                .field("data", &PrettyBytes::new(&self.data))
                .finish()
        }
    }
    impl<'a> MapData<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<MapData<'a>, Error> {
            Ok(MapData {
                last: try!(_p.read_int()),
                crc: try!(_p.read_int()),
                chunk: try!(_p.read_int()),
                data: try!(_p.read_data()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.last));
            try!(_p.write_int(self.crc));
            try!(_p.write_int(self.chunk));
            try!(_p.write_data(self.data));
            Ok(_p.written())
        }
    }

    impl fmt::Debug for ConReady {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("ConReady")
                .finish()
        }
    }
    impl ConReady {
        pub fn decode(_p: &mut Unpacker) -> Result<ConReady, Error> {
            Ok(ConReady)
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for Snap<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Snap")
                .field("tick", &self.tick)
                .field("delta_tick", &self.delta_tick)
                .field("num_parts", &self.num_parts)
                .field("part", &self.part)
                .field("crc", &self.crc)
                .field("data", &PrettyBytes::new(&self.data))
                .finish()
        }
    }
    impl<'a> Snap<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<Snap<'a>, Error> {
            Ok(Snap {
                tick: try!(_p.read_int()),
                delta_tick: try!(_p.read_int()),
                num_parts: try!(_p.read_int()),
                part: try!(_p.read_int()),
                crc: try!(_p.read_int()),
                data: try!(_p.read_data()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.tick));
            try!(_p.write_int(self.delta_tick));
            try!(_p.write_int(self.num_parts));
            try!(_p.write_int(self.part));
            try!(_p.write_int(self.crc));
            try!(_p.write_data(self.data));
            Ok(_p.written())
        }
    }

    impl fmt::Debug for SnapEmpty {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("SnapEmpty")
                .field("tick", &self.tick)
                .field("delta_tick", &self.delta_tick)
                .finish()
        }
    }
    impl SnapEmpty {
        pub fn decode(_p: &mut Unpacker) -> Result<SnapEmpty, Error> {
            Ok(SnapEmpty {
                tick: try!(_p.read_int()),
                delta_tick: try!(_p.read_int()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.tick));
            try!(_p.write_int(self.delta_tick));
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for SnapSingle<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("SnapSingle")
                .field("tick", &self.tick)
                .field("delta_tick", &self.delta_tick)
                .field("crc", &self.crc)
                .field("data", &PrettyBytes::new(&self.data))
                .finish()
        }
    }
    impl<'a> SnapSingle<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<SnapSingle<'a>, Error> {
            Ok(SnapSingle {
                tick: try!(_p.read_int()),
                delta_tick: try!(_p.read_int()),
                crc: try!(_p.read_int()),
                data: try!(_p.read_data()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.tick));
            try!(_p.write_int(self.delta_tick));
            try!(_p.write_int(self.crc));
            try!(_p.write_data(self.data));
            Ok(_p.written())
        }
    }

    impl fmt::Debug for InputTiming {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("InputTiming")
                .field("input_pred_tick", &self.input_pred_tick)
                .field("time_left", &self.time_left)
                .finish()
        }
    }
    impl InputTiming {
        pub fn decode(_p: &mut Unpacker) -> Result<InputTiming, Error> {
            Ok(InputTiming {
                input_pred_tick: try!(_p.read_int()),
                time_left: try!(_p.read_int()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.input_pred_tick));
            try!(_p.write_int(self.time_left));
            Ok(_p.written())
        }
    }

    impl fmt::Debug for RconAuthStatus {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RconAuthStatus")
                .field("auth_level", &self.auth_level)
                .field("receive_commands", &self.receive_commands)
                .finish()
        }
    }
    impl RconAuthStatus {
        pub fn decode(_p: &mut Unpacker) -> Result<RconAuthStatus, Error> {
            Ok(RconAuthStatus {
                auth_level: _p.read_int().ok(),
                receive_commands: _p.read_int().ok(),
            })
        }
    }

    impl<'a> fmt::Debug for RconLine<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RconLine")
                .field("line", &PrettyBytes::new(&self.line))
                .finish()
        }
    }
    impl<'a> RconLine<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<RconLine<'a>, Error> {
            Ok(RconLine {
                line: try!(_p.read_string()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self.line));
            Ok(_p.written())
        }
    }

    impl fmt::Debug for Ready {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Ready")
                .finish()
        }
    }
    impl Ready {
        pub fn decode(_p: &mut Unpacker) -> Result<Ready, Error> {
            Ok(Ready)
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            Ok(_p.written())
        }
    }

    impl fmt::Debug for EnterGame {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("EnterGame")
                .finish()
        }
    }
    impl EnterGame {
        pub fn decode(_p: &mut Unpacker) -> Result<EnterGame, Error> {
            Ok(EnterGame)
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for Input<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Input")
                .field("ack_snapshot", &self.ack_snapshot)
                .field("intended_tick", &self.intended_tick)
                .field("input", &self.input)
                .finish()
        }
    }
    impl<'a> Input<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<Input<'a>, Error> {
            Ok(Input {
                ack_snapshot: try!(_p.read_int()),
                intended_tick: try!(_p.read_int()),
                input: try!(_p.read_rest().map(IntegerData::from_bytes)),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.ack_snapshot));
            try!(_p.write_int(self.intended_tick));
            try!(_p.write_rest(self.input.as_bytes()));
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for RconCmd<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RconCmd")
                .field("cmd", &PrettyBytes::new(&self.cmd))
                .finish()
        }
    }
    impl<'a> RconCmd<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<RconCmd<'a>, Error> {
            Ok(RconCmd {
                cmd: try!(_p.read_string()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self.cmd));
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for RconAuth<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RconAuth")
                .field("_unused", &PrettyBytes::new(&self._unused))
                .field("password", &PrettyBytes::new(&self.password))
                .field("request_commands", &self.request_commands)
                .finish()
        }
    }
    impl<'a> RconAuth<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<RconAuth<'a>, Error> {
            Ok(RconAuth {
                _unused: try!(_p.read_string()),
                password: try!(_p.read_string()),
                request_commands: _p.read_int().ok(),
            })
        }
    }

    impl fmt::Debug for RequestMapData {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RequestMapData")
                .field("chunk", &self.chunk)
                .finish()
        }
    }
    impl RequestMapData {
        pub fn decode(_p: &mut Unpacker) -> Result<RequestMapData, Error> {
            Ok(RequestMapData {
                chunk: try!(_p.read_int()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_int(self.chunk));
            Ok(_p.written())
        }
    }

    impl fmt::Debug for Ping {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Ping")
                .finish()
        }
    }
    impl Ping {
        pub fn decode(_p: &mut Unpacker) -> Result<Ping, Error> {
            Ok(Ping)
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            Ok(_p.written())
        }
    }

    impl fmt::Debug for PingReply {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("PingReply")
                .finish()
        }
    }
    impl PingReply {
        pub fn decode(_p: &mut Unpacker) -> Result<PingReply, Error> {
            Ok(PingReply)
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for RconCmdAdd<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RconCmdAdd")
                .field("name", &PrettyBytes::new(&self.name))
                .field("help", &PrettyBytes::new(&self.help))
                .field("params", &PrettyBytes::new(&self.params))
                .finish()
        }
    }
    impl<'a> RconCmdAdd<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<RconCmdAdd<'a>, Error> {
            Ok(RconCmdAdd {
                name: try!(_p.read_string()),
                help: try!(_p.read_string()),
                params: try!(_p.read_string()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self.name));
            try!(_p.write_string(self.help));
            try!(_p.write_string(self.params));
            Ok(_p.written())
        }
    }

    impl<'a> fmt::Debug for RconCmdRemove<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RconCmdRemove")
                .field("name", &PrettyBytes::new(&self.name))
                .finish()
        }
    }
    impl<'a> RconCmdRemove<'a> {
        pub fn decode(_p: &mut Unpacker<'a>) -> Result<RconCmdRemove<'a>, Error> {
            Ok(RconCmdRemove {
                name: try!(_p.read_string()),
            })
        }
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self.name));
            Ok(_p.written())
        }
    }

    impl<'a> Info<'a> {
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self.version));
            try!(_p.write_string(self.password.expect("Info needs a password")));
            Ok(_p.written())
        }
    }
    impl RconAuthStatus {
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            assert!(self.auth_level.is_some() || self.receive_commands.is_none());
            try!(self.auth_level.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
            try!(self.receive_commands.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
            Ok(_p.written())
        }
    }
    impl<'a> RconAuth<'a> {
        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
            -> Result<&'d [u8], CapacityError>
        {
            try!(_p.write_string(self._unused));
            try!(_p.write_string(self.password));
            try!(self.request_commands.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
            Ok(_p.written())
        }
    }

}

#[derive(Clone, Copy)]
pub enum System<'a> {
    Info(system::Info<'a>),
    MapChange(system::MapChange<'a>),
    MapData(system::MapData<'a>),
    ConReady(system::ConReady),
    Snap(system::Snap<'a>),
    SnapEmpty(system::SnapEmpty),
    SnapSingle(system::SnapSingle<'a>),
    InputTiming(system::InputTiming),
    RconAuthStatus(system::RconAuthStatus),
    RconLine(system::RconLine<'a>),
    Ready(system::Ready),
    EnterGame(system::EnterGame),
    Input(system::Input<'a>),
    RconCmd(system::RconCmd<'a>),
    RconAuth(system::RconAuth<'a>),
    RequestMapData(system::RequestMapData),
    Ping(system::Ping),
    PingReply(system::PingReply),
    RconCmdAdd(system::RconCmdAdd<'a>),
    RconCmdRemove(system::RconCmdRemove<'a>),
}

impl<'a> System<'a> {
    pub fn decode(msg_id: i32, p: &mut Unpacker<'a>) -> Result<System<'a>, Error> {
        use self::system::*;
        Ok(match msg_id {
            INFO => System::Info(try!(Info::decode(p))),
            MAP_CHANGE => System::MapChange(try!(MapChange::decode(p))),
            MAP_DATA => System::MapData(try!(MapData::decode(p))),
            CON_READY => System::ConReady(try!(ConReady::decode(p))),
            SNAP => System::Snap(try!(Snap::decode(p))),
            SNAP_EMPTY => System::SnapEmpty(try!(SnapEmpty::decode(p))),
            SNAP_SINGLE => System::SnapSingle(try!(SnapSingle::decode(p))),
            INPUT_TIMING => System::InputTiming(try!(InputTiming::decode(p))),
            RCON_AUTH_STATUS => System::RconAuthStatus(try!(RconAuthStatus::decode(p))),
            RCON_LINE => System::RconLine(try!(RconLine::decode(p))),
            READY => System::Ready(try!(Ready::decode(p))),
            ENTER_GAME => System::EnterGame(try!(EnterGame::decode(p))),
            INPUT => System::Input(try!(Input::decode(p))),
            RCON_CMD => System::RconCmd(try!(RconCmd::decode(p))),
            RCON_AUTH => System::RconAuth(try!(RconAuth::decode(p))),
            REQUEST_MAP_DATA => System::RequestMapData(try!(RequestMapData::decode(p))),
            PING => System::Ping(try!(Ping::decode(p))),
            PING_REPLY => System::PingReply(try!(PingReply::decode(p))),
            RCON_CMD_ADD => System::RconCmdAdd(try!(RconCmdAdd::decode(p))),
            RCON_CMD_REMOVE => System::RconCmdRemove(try!(RconCmdRemove::decode(p))),
            _ => return Err(Error::new()),
        })
    }
    pub fn msg_id(&self) -> i32 {
        use self::system::*;
        match *self {
            System::Info(_) => INFO,
            System::MapChange(_) => MAP_CHANGE,
            System::MapData(_) => MAP_DATA,
            System::ConReady(_) => CON_READY,
            System::Snap(_) => SNAP,
            System::SnapEmpty(_) => SNAP_EMPTY,
            System::SnapSingle(_) => SNAP_SINGLE,
            System::InputTiming(_) => INPUT_TIMING,
            System::RconAuthStatus(_) => RCON_AUTH_STATUS,
            System::RconLine(_) => RCON_LINE,
            System::Ready(_) => READY,
            System::EnterGame(_) => ENTER_GAME,
            System::Input(_) => INPUT,
            System::RconCmd(_) => RCON_CMD,
            System::RconAuth(_) => RCON_AUTH,
            System::RequestMapData(_) => REQUEST_MAP_DATA,
            System::Ping(_) => PING,
            System::PingReply(_) => PING_REPLY,
            System::RconCmdAdd(_) => RCON_CMD_ADD,
            System::RconCmdRemove(_) => RCON_CMD_REMOVE,
        }
    }
    pub fn encode<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            System::Info(ref i) => i.encode(p),
            System::MapChange(ref i) => i.encode(p),
            System::MapData(ref i) => i.encode(p),
            System::ConReady(ref i) => i.encode(p),
            System::Snap(ref i) => i.encode(p),
            System::SnapEmpty(ref i) => i.encode(p),
            System::SnapSingle(ref i) => i.encode(p),
            System::InputTiming(ref i) => i.encode(p),
            System::RconAuthStatus(ref i) => i.encode(p),
            System::RconLine(ref i) => i.encode(p),
            System::Ready(ref i) => i.encode(p),
            System::EnterGame(ref i) => i.encode(p),
            System::Input(ref i) => i.encode(p),
            System::RconCmd(ref i) => i.encode(p),
            System::RconAuth(ref i) => i.encode(p),
            System::RequestMapData(ref i) => i.encode(p),
            System::Ping(ref i) => i.encode(p),
            System::PingReply(ref i) => i.encode(p),
            System::RconCmdAdd(ref i) => i.encode(p),
            System::RconCmdRemove(ref i) => i.encode(p),
        }
    }
}
impl<'a> fmt::Debug for System<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            System::Info(m) => m.fmt(f),
            System::MapChange(m) => m.fmt(f),
            System::MapData(m) => m.fmt(f),
            System::ConReady(m) => m.fmt(f),
            System::Snap(m) => m.fmt(f),
            System::SnapEmpty(m) => m.fmt(f),
            System::SnapSingle(m) => m.fmt(f),
            System::InputTiming(m) => m.fmt(f),
            System::RconAuthStatus(m) => m.fmt(f),
            System::RconLine(m) => m.fmt(f),
            System::Ready(m) => m.fmt(f),
            System::EnterGame(m) => m.fmt(f),
            System::Input(m) => m.fmt(f),
            System::RconCmd(m) => m.fmt(f),
            System::RconAuth(m) => m.fmt(f),
            System::RequestMapData(m) => m.fmt(f),
            System::Ping(m) => m.fmt(f),
            System::PingReply(m) => m.fmt(f),
            System::RconCmdAdd(m) => m.fmt(f),
            System::RconCmdRemove(m) => m.fmt(f),
        }
    }
}

