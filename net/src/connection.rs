use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use buffer;
use protocol::ChunksIter;
use protocol::ConnectedPacket;
use protocol::ConnectedPacketType;
use protocol::ControlPacket;
use protocol::MAX_PACKETSIZE;
use protocol::MAX_PAYLOAD;
use protocol::Packet;
use protocol;
use std::collections::VecDeque;
use std::mem;

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

impl State {
    pub fn assert_online(&mut self) -> &mut OnlineState {
        match *self {
            State::Online(ref mut s) => s,
            _ => panic!("state not online"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResendChunk {
    sequence: Sequence,
    data: ArrayVec<[u8; 2048]>,
}

impl ResendChunk {
    fn new(sequence: Sequence, data: &[u8]) -> ResendChunk {
        let result = ResendChunk {
            sequence: sequence,
            data: data.iter().cloned().collect(),
        };
        assert!(result.data.len() == data.len(), "overlong resend packet {}", data.len());
        result
    }
}

pub struct ReceivePacket<'a> {
    type_: ReceivePacketType<'a>,
}

impl<'a> Clone for ReceivePacket<'a> {
    fn clone(&self) -> ReceivePacket<'a> {
        ReceivePacket {
            type_: self.type_.clone(),
        }
    }
}

impl<'a> ReceivePacket<'a> {
    fn connless(data: &'a [u8]) -> ReceivePacket<'a> {
        ReceivePacket {
            type_: ReceivePacketType::Connless(data, false),
        }
    }
    fn connected(online_state: &'a mut OnlineState, data: &'a [u8]) -> ReceivePacket<'a> {
        ReceivePacket {
            type_: ReceivePacketType::Connected(ReceiveChunks {
                online_state: Some(online_state),
                chunks: ChunksIter::new(data),
            }),
        }
    }
}

#[derive(Clone)]
enum ReceivePacketType<'a> {
    // Connless(data, done)
    Connless(&'a [u8], bool),
    Connected(ReceiveChunks<'a>),
}

impl<'a> Drop for ReceivePacket<'a> {
    fn drop(&mut self) {
        for _ in self { }
    }
}

impl<'a> Iterator for ReceivePacket<'a> {
    type Item = ReceiveChunk<'a>;
    fn next(&mut self) -> Option<ReceiveChunk<'a>> {
        match self.type_ {
            ReceivePacketType::Connless(data, ref mut done) => {
                let done = mem::replace(done, true);
                if !done {
                    Some(ReceiveChunk::Connless(data))
                } else {
                    None
                }
            }
            ReceivePacketType::Connected(ref mut chunks) => {
                chunks.next()
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.clone().count();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for ReceivePacket<'a> { }

struct ReceiveChunks<'a> {
    online_state: Option<&'a mut OnlineState>,
    chunks: ChunksIter<'a>,
}

impl<'a> Iterator for ReceiveChunks<'a> {
    type Item = ReceiveChunk<'a>;
    fn next(&mut self) -> Option<ReceiveChunk<'a>> {
        self.chunks.next().map(|c| {
            if let Some(ref v) = c.vital {
                // TODO: Update internal ack variable
                unimplemented!();
            }
            ReceiveChunk::Connected(c.data, c.vital.is_some())
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chunks.size_hint()
    }
}

impl<'a> ExactSizeIterator for ReceiveChunks<'a> { }

impl<'a> Clone for ReceiveChunks<'a> {
    fn clone(&self) -> ReceiveChunks<'a> {
        ReceiveChunks {
            online_state: None,
            chunks: self.chunks.clone(),
        }
    }
}

pub enum ReceiveChunk<'a> {
    Connless(&'a [u8]),
    // Connected(data, vital)
    Connected(&'a [u8], bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OnlineState {
    ack: Sequence,
    sequence: Sequence,
    request_resend: bool,
    packet_num_chunks: u8,
    packet: ArrayVec<[u8; 2048]>,
    resend_queue: VecDeque<ResendChunk>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Sequence {
    seq: u16, // u10
}

impl Sequence {
    fn new() -> Sequence {
        Default::default()
    }
    fn to_u16(self) -> u16 {
        self.seq
    }
    fn next(&mut self) -> Sequence {
        let result = *self;
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
        let data = match packet.write(&mut self.compression_buffer[..], &mut self.buffer[..]) {
            Ok(d) => d,
            Err(protocol::Error::Capacity(_)) => unreachable!("too short buffer provided"),
            Err(protocol::Error::TooLongData) => return Err(Error::TooLongData),
        };
        try!(cb.send(data));
        Ok(())
    }
    fn compression_buffer(&mut self) -> &mut [u8] {
        &mut self.compression_buffer[..]
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
            ack: online.ack.to_u16(),
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
            if !resend {
                online.resend_queue.push_back(ResendChunk::new(sequence, buffer));
            }
            Some((sequence.to_u16(), resend))
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
    pub fn send_connless<CB: Callback>(&mut self, cb: &mut CB, data: &[u8])
        -> Result<(), Error<CB::Error>>
    {
        self.state.assert_online();
        self.builder.send(cb, Packet::Connless(data))
    }
    pub fn feed<'a, CB: Callback>(&'a mut self, cb: &mut CB, data: &'a [u8])
        -> Option<ReceivePacket<'a>>
    {
        if data.len() > protocol::MAX_PACKETSIZE {
            // TODO: Warn?
            return None;
        }
        let buf = self.builder.compression_buffer();
        // TODO: Warn?
        let packet = unwrap_or_return!(Packet::read(data, buf));

        let connected = match packet {
            Packet::Connless(data) => return Some(ReceivePacket::connless(data)),
            Packet::Connected(c) => c,
        };
        let ConnectedPacket { ack, type_ } = connected;
        // TODO: do something with ack
        let _ = ack;
        unimplemented!();
    }
}
