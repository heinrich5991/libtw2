use arrayvec::ArrayVec;
use common::Buffer;
use common::buffer::SliceBuffer;
use protocol::ConnectedPacket;
use protocol::ConnectedPacketType;
use protocol::ControlPacket;
use protocol::MAX_PACKETSIZE;
use protocol::MAX_PAYLOAD;
use protocol::Packet;
use protocol;
use std::collections::VecDeque;

pub trait Callback {
    type Error;
    fn send(&mut self, buffer: &[u8]) -> Result<(), Self::Error>;
}

pub struct Connection {
    state: State,
    builder: PacketBuilder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Unconnected,
    Connecting,
    Pending,
    Online(OnlineState),
    Disconnected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResendPacket {
    _unused: (),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OnlineState {
    ack: Sequence,
    sequence: Sequence,
    request_resend: bool,
    packet_num_chunks: u8,
    packet: ArrayVec<[u8; 2048]>,
    resend_queue: VecDeque<ResendPacket>,
}

impl State {
    pub fn assert_online(&mut self) -> &mut OnlineState {
        match *self {
            State::Online(ref mut s) => s,
            _ => panic!("state not online"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Sequence {
    seq: u16, // u10
}

impl Sequence {
    fn new() -> Sequence {
        Default::default()
    }
    fn get(&self) -> u16 {
        self.seq
    }
    fn next(&mut self) -> u16 {
        let result = self.seq;
        self.seq = (self.seq + 1) % (1 << protocol::SEQUENCE_BITS);
        result
    }
}

struct PacketBuilder {
    compression_buffer: [u8; MAX_PACKETSIZE],
    buffer: [u8; MAX_PACKETSIZE],
}

impl PacketBuilder {
    fn new() -> PacketBuilder {
        PacketBuilder {
            compression_buffer: [0; MAX_PACKETSIZE],
            buffer: [0; MAX_PACKETSIZE],
        }
    }
    fn send<CB: Callback>(&mut self, cb: &mut CB, packet: Packet)
        -> Result<(), Error<CB::Error>>
    {
        let compression_buffer = &mut SliceBuffer::new(&mut self.compression_buffer);
        let buffer = &mut SliceBuffer::new(&mut self.buffer);
        match packet.write(compression_buffer, buffer) {
            Ok(()) => {},
            Err(protocol::Error::Capacity(_)) => unreachable!("too short buffer provided"),
            Err(protocol::Error::TooLongData) => return Err(Error::TooLongData),
        }
        try!(cb.send(buffer));
        Ok(())
    }
}

pub enum Error<CE> {
    TooLongData,
    Callback(CE),
}

impl<CE> From<CE> for Error<CE> {
    fn from(e: CE) -> Error<CE> {
        Error::Callback(e)
    }
}

impl<CE> Error<CE> {
    fn unwrap_callback(self) -> CE {
        match self {
            Error::TooLongData => panic!("too long data"),
            Error::Callback(e) => e,
        }
    }
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            state: State::Unconnected,
            builder: PacketBuilder::new(),
        }
    }
    pub fn reset(&mut self) {
        assert!(self.state == State::Disconnected);
        *self = Connection::new();
    }
    pub fn connect<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        assert!(self.state == State::Unconnected);
        self.builder.send(cb, Packet::Connected(ConnectedPacket {
            ack: 0,
            type_: ConnectedPacketType::Control(ControlPacket::Connect),
        })).map_err(|e| e.unwrap_callback())
    }
    pub fn flush<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        let online = self.state.assert_online();
        if online.packet_num_chunks == 0 && online.request_resend == false {
            return Ok(());
        }
        let result = self.builder.send(cb, Packet::Connected(ConnectedPacket {
            ack: online.ack.get(),
            type_: ConnectedPacketType::Chunks(
                online.request_resend,
                online.packet_num_chunks,
                &online.packet,
            ),
        })).map_err(|e| e.unwrap_callback());
        online.request_resend = false;
        online.packet_num_chunks = 0;
        online.packet.clear();
        result
    }
    fn queue(&mut self, buffer: &[u8], vital: bool, resend: bool) {
        let online = self.state.assert_online();
        let vital = if vital {
            let sequence = online.sequence.next();
            // TODO: Put packet into resend buffer.
            Some((sequence, resend))
        } else {
            None
        };
        protocol::write_chunk(buffer, vital, &mut online.packet).unwrap();
    }
    pub fn send<CB: Callback>(&mut self, cb: &mut CB, buffer: &[u8], vital: bool)
        -> Result<(), Error<CB::Error>>
    {
        let len = self.state.assert_online().packet.len();
        let result;
        if buffer.len() > MAX_PAYLOAD {
            return Err(Error::TooLongData);
        }
        if len + protocol::chunk_header_size(vital) + buffer.len() > MAX_PAYLOAD {
            result = self.flush(cb).map_err(Error::from);
        } else {
            result = Ok(());
        }
        self.queue(buffer, vital, false);
        result
    }
    pub fn send_connless<CB: Callback>(&mut self, cb: &mut CB, buffer: &[u8])
        -> Result<(), Error<CB::Error>>
    {
        self.state.assert_online();
        self.builder.send(cb, Packet::Connless(buffer))
    }
    pub fn feed<CB: Callback>(&mut self, cb: &mut CB, buffer: &[u8]) {
        unimplemented!();
    }
}
