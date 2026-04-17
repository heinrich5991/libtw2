use crate::protocol7 as protocol;
use crate::protocol7::ChunksIter;
use crate::protocol7::ConnectedPacket;
use crate::protocol7::ConnectedPacketType;
use crate::protocol7::ConnlessPacket;
use crate::protocol7::ControlPacket;
use crate::protocol7::Packet;
use crate::protocol7::Token;
use crate::protocol7::MAX_PACKETSIZE;
use crate::protocol7::MAX_PAYLOAD;
use crate::protocol7::TOKEN_NONE;
use crate::Timeout;
use crate::Timestamp;
use arrayvec::ArrayVec;
use libtw2_buffer::with_buffer;
use libtw2_buffer::Buffer;
use libtw2_buffer::BufferRef;
use libtw2_warn::Warn;
use std::cmp;
use std::collections::VecDeque;
use std::iter;
use std::time::Duration;

// TODO: Implement receive timeout.
// TODO: Don't allow for unbounded backlog of vital messages.

pub trait Callback {
    type Error;
    fn secure_random(&mut self, buffer: &mut [u8]);
    fn send(&mut self, buffer: &[u8]) -> Result<(), Self::Error>;
    fn time(&mut self) -> Timestamp;
}

#[derive(Debug)]
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
    pub fn unwrap_callback(self) -> CE {
        match self {
            Error::TooLongData => panic!("too long data"),
            Error::Callback(e) => e,
        }
    }
}

#[derive(Debug)]
pub enum Warning {
    ConnlessResponseTokenMismatch,
    ConnlessTokenMismatch,
    Packet(protocol::Warning),
    Read(protocol::PacketReadError),
    TokenMismatch,
    Unexpected,
}

trait TimeoutExt {
    fn set<CB: Callback>(&mut self, cb: &mut CB, value: Duration);
    fn has_triggered_level<CB: Callback>(&self, cb: &mut CB) -> bool;
    fn has_triggered_edge<CB: Callback>(&mut self, cb: &mut CB) -> bool;
}

impl TimeoutExt for Timeout {
    fn set<CB: Callback>(&mut self, cb: &mut CB, value: Duration) {
        *self = Timeout::active(cb.time() + value);
    }
    fn has_triggered_level<CB: Callback>(&self, cb: &mut CB) -> bool {
        self.to_opt().map(|time| time <= cb.time()).unwrap_or(false)
    }
    fn has_triggered_edge<CB: Callback>(&mut self, cb: &mut CB) -> bool {
        let triggered = self.has_triggered_level(cb);
        if triggered {
            *self = Timeout::inactive();
        }
        triggered
    }
}

pub struct Connection {
    state: State,
    send: Timeout,
    builder: PacketBuilder,
}

#[derive(Clone, Debug)]
enum State {
    Unconnected,
    Token(TokenState),
    PendingConnect(PendingConnectState),
    Connecting(ConnectingState),
    Pending(PendingState),
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
    pub fn own_token(&self) -> Option<Token> {
        use self::State::*;
        match *self {
            Unconnected => None,
            Token(ref token) => Some(token.own_token),
            PendingConnect(ref pending_connect) => Some(pending_connect.own_token),
            Connecting(ref connecting) => Some(connecting.own_token),
            Pending(ref pending) => Some(pending.own_token),
            Online(ref online) => Some(online.own_token),
            Disconnected => None,
        }
    }
    pub fn their_token(&self) -> Option<Token> {
        use self::State::*;
        match *self {
            Unconnected => None,
            Token(_) => None,
            PendingConnect(_) => None,
            Connecting(ref connecting) => Some(connecting.their_token),
            Pending(ref pending) => Some(pending.their_token),
            Online(ref online) => Some(online.their_token),
            Disconnected => None,
        }
    }
}

#[derive(Clone, Debug)]
struct ResendChunk {
    next_send: Timeout,
    sequence: Sequence,
    data: ArrayVec<[u8; 2048]>,
}

impl ResendChunk {
    fn new<CB: Callback>(cb: &mut CB, sequence: Sequence, data: &[u8]) -> ResendChunk {
        let mut result = ResendChunk {
            next_send: Timeout::inactive(),
            sequence: sequence,
            data: data.iter().cloned().collect(),
        };
        assert!(
            result.data.len() == data.len(),
            "overlong resend packet {}",
            data.len()
        );
        result.start_timeout(cb);
        result
    }
    fn start_timeout<CB: Callback>(&mut self, cb: &mut CB) {
        self.next_send.set(cb, Duration::from_millis(1_000));
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
    fn none() -> ReceivePacket<'a> {
        ReceivePacket {
            type_: ReceivePacketType::None,
        }
    }
    fn ready() -> ReceivePacket<'a> {
        ReceivePacket {
            type_: ReceivePacketType::Ready(iter::once(())),
        }
    }
    fn connless(data: &'a [u8]) -> ReceivePacket<'a> {
        ReceivePacket {
            type_: ReceivePacketType::Connless(iter::once(data)),
        }
    }
    fn connected<W>(
        warn: &mut W,
        online: &mut OnlineState,
        num_chunks: u8,
        data: &'a [u8],
    ) -> ReceivePacket<'a>
    where
        W: Warn<Warning>,
    {
        let chunks_iter = ChunksIter::new(data, num_chunks);
        let ack = online.ack.clone();
        let mut iter = chunks_iter.clone();
        while let Some(c) = iter.next_warn(&mut w(warn)) {
            if let Some((sequence, resend)) = c.vital {
                let _ = resend;
                if online.ack.update(Sequence::from_u16(sequence)) != SequenceOrdering::Current {
                    online.request_resend = true;
                }
            }
        }
        ReceivePacket {
            type_: ReceivePacketType::Connected(ReceiveChunks {
                ack: ack,
                chunks: chunks_iter,
            }),
        }
    }
    fn disconnect(reason: &'a [u8]) -> ReceivePacket<'a> {
        ReceivePacket {
            type_: ReceivePacketType::Close(iter::once(reason)),
        }
    }
}

#[derive(Clone)]
enum ReceivePacketType<'a> {
    None,
    Connless(iter::Once<&'a [u8]>),
    Connected(ReceiveChunks<'a>),
    Ready(iter::Once<()>),
    Close(iter::Once<&'a [u8]>),
}

impl<'a> Iterator for ReceivePacket<'a> {
    type Item = ReceiveChunk<'a>;
    fn next(&mut self) -> Option<ReceiveChunk<'a>> {
        match self.type_ {
            ReceivePacketType::None => None,
            ReceivePacketType::Ready(ref mut once) => once.next().map(|()| ReceiveChunk::Ready),
            ReceivePacketType::Connless(ref mut once) => once.next().map(ReceiveChunk::Connless),
            ReceivePacketType::Connected(ref mut chunks) => chunks.next(),
            ReceivePacketType::Close(ref mut once) => once.next().map(ReceiveChunk::Disconnect),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.clone().count();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for ReceivePacket<'a> {}

#[derive(Clone)]
struct ReceiveChunks<'a> {
    ack: Sequence,
    chunks: ChunksIter<'a>,
}

impl<'a> Iterator for ReceiveChunks<'a> {
    type Item = ReceiveChunk<'a>;
    fn next(&mut self) -> Option<ReceiveChunk<'a>> {
        self.chunks.next().and_then(|c| {
            if let Some((sequence, resend)) = c.vital {
                let _ = resend;
                if self.ack.update(Sequence::from_u16(sequence)) != SequenceOrdering::Current {
                    return self.next();
                }
            }
            Some(ReceiveChunk::Connected(c.data, c.vital.is_some()))
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.clone().count();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for ReceiveChunks<'a> {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReceiveChunk<'a> {
    Connless(&'a [u8]),
    // Connected(data, vital)
    Connected(&'a [u8], bool),
    Ready,
    Disconnect(&'a [u8]),
}

#[derive(Clone, Debug)]
struct TokenState {
    own_token: Token,
}

impl TokenState {
    fn new(own_token: Token) -> TokenState {
        TokenState {
            own_token: own_token,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingConnectState {
    own_token: Token,
}

impl PendingConnectState {
    fn new(own_token: Token) -> PendingConnectState {
        PendingConnectState {
            own_token: own_token,
        }
    }
}

#[derive(Clone, Debug)]
struct ConnectingState {
    own_token: Token,
    their_token: Token,
}

impl ConnectingState {
    fn new(own_token: Token, their_token: Token) -> ConnectingState {
        ConnectingState {
            own_token: own_token,
            their_token: their_token,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingState {
    own_token: Token,
    their_token: Token,
}

impl PendingState {
    fn new(own_token: Token, their_token: Token) -> PendingState {
        PendingState {
            own_token: own_token,
            their_token: their_token,
        }
    }
}

#[derive(Clone, Debug)]
struct OnlineState {
    // `own_token` is included in every message from the other peer in order to
    // protect against IP spoofing.
    own_token: Token,
    // `their_token` is included in every message to the other peer in order to
    // protect against IP spoofing.
    their_token: Token,
    // `ack` is the vital chunk from the peer we want to acknowledge.
    ack: Sequence,
    // `sequence` is the vital chunk from us that the peer acknowledged.
    sequence: Sequence,
    request_resend: bool,
    // `packet` contains all the queued chunks, `packet_nonvital` only the
    // non-vital ones. This is important for resending.
    packet: PacketContents,
    packet_nonvital: PacketContents,
    // This contains the unacked chunks that we sent, starting from the most
    // recently sent chunk.
    resend_queue: VecDeque<ResendChunk>,
}

impl OnlineState {
    fn new(own_token: Token, their_token: Token) -> OnlineState {
        OnlineState {
            own_token: own_token,
            their_token: their_token,
            ack: Sequence::new(),
            sequence: Sequence::new(),
            request_resend: false,
            packet: PacketContents::new(),
            packet_nonvital: PacketContents::new(),
            resend_queue: VecDeque::new(),
        }
    }
    fn can_send(&self) -> bool {
        self.packet.num_chunks != 0 || self.request_resend
    }
    fn ack_chunks(&mut self, ack: Sequence) {
        let index = self
            .resend_queue
            .iter()
            .position(|chunk| chunk.sequence == ack);
        index.map(|i| self.resend_queue.truncate(i));
    }
    fn flush<CB: Callback>(
        &mut self,
        cb: &mut CB,
        builder: &mut PacketBuilder,
    ) -> Result<(), CB::Error> {
        if !self.can_send() {
            return Ok(());
        }
        let result = builder
            .send(
                cb,
                Packet::Connected(ConnectedPacket {
                    token: self.their_token,
                    ack: self.ack.to_u16(),
                    type_: ConnectedPacketType::Chunks(
                        self.request_resend,
                        self.packet.num_chunks,
                        &self.packet.data,
                    ),
                }),
            )
            .map_err(|e| e.unwrap_callback());
        self.request_resend = false;
        self.packet.clear();
        self.packet_nonvital.clear();
        result
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PacketContents {
    num_chunks: u8,
    data: ArrayVec<[u8; 2048]>,
}

impl PacketContents {
    fn new() -> PacketContents {
        PacketContents {
            num_chunks: 0,
            data: ArrayVec::new(),
        }
    }
    fn write_chunk(&mut self, data: &[u8], vital: Option<(u16, bool)>) {
        protocol::write_chunk(data, vital, &mut self.data).unwrap();
        self.num_chunks += 1;
    }
    fn can_fit_chunk(&self, data: &[u8], vital: bool) -> bool {
        // current size + chunk header + chunk length
        self.data.len() + protocol::chunk_header_size(vital) + data.len() <= MAX_PAYLOAD
    }
    fn clear(&mut self) {
        *self = PacketContents::new();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Sequence {
    seq: u16, // u10
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SequenceOrdering {
    Past,
    Current,
    Future,
}

impl Sequence {
    fn new() -> Sequence {
        Default::default()
    }
    fn from_u16(seq: u16) -> Sequence {
        assert!(seq < protocol::SEQUENCE_MODULUS);
        Sequence { seq: seq }
    }
    fn to_u16(self) -> u16 {
        self.seq
    }
    fn next(&mut self) -> Sequence {
        self.seq = (self.seq + 1) % protocol::SEQUENCE_MODULUS;
        *self
    }
    fn update(&mut self, other: Sequence) -> SequenceOrdering {
        let mut next_self = *self;
        next_self.next();
        let result = next_self.compare(other);
        if result == SequenceOrdering::Current {
            *self = next_self;
        }
        result
    }
    /// Returns what `other` is in relation to `self`.
    fn compare(self, other: Sequence) -> SequenceOrdering {
        let half = protocol::SEQUENCE_MODULUS / 2;
        let less;
        match self.seq.cmp(&other.seq) {
            cmp::Ordering::Less => less = other.seq - self.seq < half,
            cmp::Ordering::Greater => less = self.seq - other.seq > half,
            cmp::Ordering::Equal => return SequenceOrdering::Current,
        }
        if less {
            SequenceOrdering::Future
        } else {
            SequenceOrdering::Past
        }
    }
}

struct PacketBuilder {
    buffer: [u8; MAX_PACKETSIZE],
}

impl PacketBuilder {
    fn new() -> PacketBuilder {
        PacketBuilder {
            buffer: [0; MAX_PACKETSIZE],
        }
    }
    fn send<CB: Callback>(&mut self, cb: &mut CB, packet: Packet) -> Result<(), Error<CB::Error>> {
        let data = match packet.write(&mut self.buffer[..]) {
            Ok(d) => d,
            Err(protocol::Error::Capacity(_)) => unreachable!("too short buffer provided"),
            Err(protocol::Error::TooLongData) => return Err(Error::TooLongData),
        };
        cb.send(data)?;
        Ok(())
    }
}

struct WarnCallback<'a, W: Warn<Warning> + 'a> {
    warn: &'a mut W,
}

fn w<W: Warn<Warning>>(warn: &mut W) -> WarnCallback<'_, W> {
    WarnCallback { warn: warn }
}

impl<'a, W: Warn<Warning>> Warn<protocol::Warning> for WarnCallback<'a, W> {
    fn warn(&mut self, warning: protocol::Warning) {
        self.warn.warn(Warning::Packet(warning))
    }
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            state: State::Unconnected,
            send: Timeout::inactive(),
            builder: PacketBuilder::new(),
        }
    }
    pub fn reset(&mut self) {
        assert!(matches!(self.state, State::Disconnected));
        *self = Connection::new();
    }
    pub fn is_unconnected(&self) -> bool {
        matches!(self.state, State::Unconnected)
    }
    pub fn needs_tick(&self) -> Timeout {
        match self.state {
            State::Unconnected | State::Disconnected => return Timeout::inactive(),
            _ => {}
        }
        let resends = match self.state {
            State::Online(ref online) => online
                .resend_queue
                .back()
                .map(|r| r.next_send)
                .unwrap_or_default(),
            _ => Timeout::inactive(),
        };
        cmp::min(self.send, resends)
    }
    pub fn connect<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        assert!(matches!(self.state, State::Unconnected));
        self.state = State::Token(TokenState::new(Token::random(|b| cb.secure_random(b))));
        self.tick_action(cb)?;
        Ok(())
    }
    pub fn disconnect<CB: Callback>(
        &mut self,
        cb: &mut CB,
        reason: &[u8],
    ) -> Result<(), CB::Error> {
        if let State::Disconnected = self.state {
            assert!(
                false,
                "Can't call disconnect on an already disconnected connection"
            );
        }
        assert!(
            reason.iter().all(|&b| b != 0),
            "reason must not contain NULs"
        );
        let result = self.send_control(cb, ControlPacket::Close(reason));
        self.state = State::Disconnected;
        result
    }
    fn resend<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        let online = self.state.assert_online();
        if online.resend_queue.is_empty() {
            return Ok(());
        }
        online.packet = online.packet_nonvital.clone();
        let mut i = 0;
        for chunk in &mut online.resend_queue {
            chunk.start_timeout(cb);
        }
        while i < online.resend_queue.len() {
            let can_fit;
            {
                let chunk = &online.resend_queue[online.resend_queue.len() - i - 1];
                can_fit = online.packet.can_fit_chunk(&chunk.data, true);
                if can_fit {
                    let vital = (chunk.sequence.to_u16(), true);
                    online.packet.write_chunk(&chunk.data, Some(vital));
                    i += 1;
                }
            }
            if !can_fit {
                self.send.set(cb, Duration::from_millis(500));
                online.flush(cb, &mut self.builder)?;
            }
        }
        Ok(())
    }
    pub fn flush<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        self.send.set(cb, Duration::from_millis(500));
        self.state.assert_online().flush(cb, &mut self.builder)
    }
    fn queue<CB: Callback>(&mut self, cb: &mut CB, buffer: &[u8], vital: bool) {
        let online = self.state.assert_online();
        let vital = if vital {
            let sequence = online.sequence.next();
            online
                .resend_queue
                .push_front(ResendChunk::new(cb, sequence, buffer));
            Some((sequence.to_u16(), false))
        } else {
            None
        };
        if vital.is_none() {
            online.packet_nonvital.write_chunk(buffer, vital);
        }
        online.packet.write_chunk(buffer, vital)
    }
    pub fn send<CB: Callback>(
        &mut self,
        cb: &mut CB,
        buffer: &[u8],
        vital: bool,
    ) -> Result<(), Error<CB::Error>> {
        let result;
        {
            let online = self.state.assert_online();
            if buffer.len() > MAX_PAYLOAD {
                return Err(Error::TooLongData);
            }
            if !online.packet.can_fit_chunk(buffer, vital) {
                result = online.flush(cb, &mut self.builder).map_err(Error::from);
            } else {
                result = Ok(());
            }
        }
        self.queue(cb, buffer, vital);
        result
    }
    pub fn send_connless<CB: Callback>(
        &mut self,
        cb: &mut CB,
        data: &[u8],
    ) -> Result<(), Error<CB::Error>> {
        let online = self.state.assert_online();
        self.send.set(cb, Duration::from_millis(500));
        self.builder.send(
            cb,
            Packet::Connless(ConnlessPacket {
                token: online.their_token,
                response_token: online.own_token,
                payload: data,
            }),
        )
    }
    fn send_control<CB: Callback>(
        &mut self,
        cb: &mut CB,
        control: ControlPacket,
    ) -> Result<(), CB::Error> {
        self.send_control_with_token(cb, control, self.state.their_token().unwrap_or(TOKEN_NONE))
    }
    fn send_control_with_token<CB: Callback>(
        &mut self,
        cb: &mut CB,
        control: ControlPacket,
        token: Token,
    ) -> Result<(), CB::Error> {
        let ack = match self.state {
            State::Online(ref mut online) => online.ack.to_u16(),
            _ => 0,
        };
        self.builder
            .send(
                cb,
                Packet::Connected(ConnectedPacket {
                    token: token,
                    ack: ack,
                    type_: ConnectedPacketType::Control(control),
                }),
            )
            .map_err(|e| e.unwrap_callback())
    }
    pub fn tick<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        let do_resend = match self.state {
            State::Online(ref online) => {
                // WARN?
                online
                    .resend_queue
                    .back()
                    .map(|c| c.next_send.has_triggered_level(cb))
                    .unwrap_or(false)
            }
            _ => false,
        };
        if do_resend {
            self.resend(cb)
        } else if self.send.has_triggered_edge(cb) {
            self.tick_action(cb)
        } else {
            Ok(())
        }
    }
    fn tick_action<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        let control = match self.state {
            State::Token(ref token) => ControlPacket::Token(token.own_token),
            State::Connecting(ref connecting) => ControlPacket::Connect(connecting.own_token),
            State::Pending(_) => ControlPacket::Accept,
            State::Online(ref mut online) => {
                if online.can_send() {
                    // TODO: Warn if this happens on reliable networks.
                    self.send.set(cb, Duration::from_millis(500));
                    return online.flush(cb, &mut self.builder);
                }
                ControlPacket::KeepAlive
            }
            _ => return Ok(()),
        };
        self.send.set(cb, Duration::from_millis(500));
        self.send_control(cb, control)
    }
    /// Notifies the connection of incoming data.
    ///
    /// `buffer` must have at least size `MAX_PAYLOAD`.
    pub fn feed<'a, B, CB, W>(
        &mut self,
        cb: &mut CB,
        warn: &mut W,
        data: &'a [u8],
        buf: B,
    ) -> (ReceivePacket<'a>, Result<(), CB::Error>)
    where
        B: Buffer<'a>,
        CB: Callback,
        W: Warn<Warning>,
    {
        with_buffer(buf, |b| self.feed_impl(cb, warn, data, b))
    }

    pub fn feed_impl<'d, 's, CB, W>(
        &mut self,
        cb: &mut CB,
        warn: &mut W,
        data: &'d [u8],
        mut buffer: BufferRef<'d, 's>,
    ) -> (ReceivePacket<'d>, Result<(), CB::Error>)
    where
        CB: Callback,
        W: Warn<Warning>,
    {
        let none = (ReceivePacket::none(), Ok(()));
        {
            use protocol::ConnectedPacketType::*;
            use protocol::ControlPacket::*;

            let packet = match Packet::read(&mut w(warn), data, &mut buffer) {
                Ok(p) => p,
                Err(e) => {
                    warn.warn(Warning::Read(e));
                    return none;
                }
            };

            let connected = match packet {
                Packet::Connless(ConnlessPacket {
                    token,
                    response_token,
                    payload,
                }) => {
                    if Some(token) != self.state.own_token() {
                        warn.warn(Warning::ConnlessTokenMismatch);
                        return none;
                    }
                    if Some(response_token) != self.state.their_token() {
                        warn.warn(Warning::ConnlessResponseTokenMismatch);
                        return none;
                    }
                    return (ReceivePacket::connless(payload), Ok(()));
                }
                Packet::Connected(c) => c,
            };
            let ConnectedPacket { token, ack, type_ } = connected;

            let mut expected_token = self.state.own_token().unwrap_or(TOKEN_NONE);
            // Allow unauthenticated TOKEN requests before the first
            // authenticated message, even if we've replied to a TOKEN request
            // before.
            //
            // TODO: test this
            if matches!(type_, Control(Token(_)))
                && matches!(self.state, State::PendingConnect(_))
                && token == TOKEN_NONE
            {
                expected_token = TOKEN_NONE;
            }
            if token != expected_token {
                warn.warn(Warning::TokenMismatch);
                return none;
            }

            // TODO: Check ack for sanity.
            if let State::Online(ref mut online) = self.state {
                online.ack_chunks(Sequence::from_u16(ack));
            }

            match type_ {
                Chunks(request_resend, num_chunks, chunks) => {
                    let _ = num_chunks;
                    if let State::Pending(ref pending) = self.state {
                        self.state =
                            State::Online(OnlineState::new(pending.own_token, pending.their_token));
                    }
                    let result;
                    if request_resend {
                        if let State::Online(_) = self.state {
                            result = self.resend(cb);
                        } else {
                            result = Ok(());
                        }
                    } else {
                        result = Ok(())
                    }
                    match self.state {
                        State::Online(ref mut online) => {
                            return (
                                ReceivePacket::connected(warn, online, num_chunks, chunks),
                                result,
                            );
                        }
                        State::Pending(_) => unreachable!(),
                        // WARN: packet received while not online.
                        _ => return none,
                    }
                }
                Control(Token(their_token)) => {
                    if let State::Unconnected = self.state {
                        use self::Token;
                        self.state =
                            State::PendingConnect(PendingConnectState::new(Token::random(|b| {
                                cb.secure_random(b)
                            })));
                    }

                    match self.state {
                        State::Unconnected => unreachable!(),
                        State::PendingConnect(ref pending_connect) => {
                            return (
                                ReceivePacket::none(),
                                self.send_control_with_token(
                                    cb,
                                    ControlPacket::Token(pending_connect.own_token),
                                    their_token,
                                ),
                            );
                        }
                        State::Token(ref token) => {
                            self.state = State::Connecting(ConnectingState::new(
                                token.own_token,
                                their_token,
                            ));
                            // Fall through to tick.
                        }
                        _ => return none,
                    }
                }
                Control(KeepAlive) => return none,
                Control(Connect(their_token)) => {
                    if let State::PendingConnect(ref pending_connect) = self.state {
                        self.state = State::Pending(PendingState::new(
                            pending_connect.own_token,
                            their_token,
                        ));
                        // Fall through to tick.
                    } else {
                        return none;
                    }
                }
                Control(Accept) => {
                    if let State::Connecting(ref connecting) = self.state {
                        self.state = State::Online(OnlineState::new(
                            connecting.own_token,
                            connecting.their_token,
                        ));
                        return (ReceivePacket::ready(), Ok(()));
                    } else {
                        return none;
                    }
                }
                Control(Close(reason)) => {
                    self.state = State::Disconnected;
                    return (ReceivePacket::disconnect(reason), Ok(()));
                }
            }
        }
        // Fall-through from `Control(Connect)`
        (ReceivePacket::none(), self.tick_action(cb))
    }
}

#[cfg(test)]
mod test {
    use super::Callback;
    use super::Connection;
    use super::ReceiveChunk;
    use super::Sequence;
    use super::SequenceOrdering;
    use crate::protocol7 as protocol;
    use crate::Timestamp;
    use hexdump::hexdump;
    use itertools::Itertools;
    use libtw2_warn::Panic;
    use std::collections::VecDeque;
    use void::ResultVoidExt;
    use void::Void;

    #[test]
    fn sequence_compare() {
        use super::SequenceOrdering::*;

        fn cmp(a: Sequence, b: Sequence) -> SequenceOrdering {
            Sequence::compare(a, b)
        }
        let default = Sequence::new();
        let first = Sequence::from_u16(0);
        let mid = Sequence::from_u16(protocol::SEQUENCE_MODULUS / 2);
        let end = Sequence::from_u16(protocol::SEQUENCE_MODULUS - 1);
        assert_eq!(cmp(default, first), Current);
        assert_eq!(cmp(first, mid), Past);
        assert_eq!(cmp(first, end), Past);
        assert_eq!(cmp(mid, first), Past);
        assert_eq!(cmp(mid, end), Future);
        assert_eq!(cmp(end, first), Future);
        assert_eq!(cmp(end, mid), Past);
    }

    #[test]
    fn establish_connection() {
        #[derive(Default)]
        struct Cb {
            sent: VecDeque<Vec<u8>>,
            random_value: Option<[u8; 4]>,
        }
        impl Callback for Cb {
            type Error = Void;
            fn secure_random(&mut self, buffer: &mut [u8]) {
                if buffer.len() != 4 {
                    unimplemented!();
                }
                buffer.copy_from_slice(&self.random_value.take().unwrap());
            }
            fn send(&mut self, data: &[u8]) -> Result<(), Void> {
                self.sent.push_back(data.to_owned());
                Ok(())
            }
            fn time(&mut self) -> Timestamp {
                Timestamp::from_secs_since_epoch(0)
            }
        }
        let mut buffer = [0; protocol::MAX_PACKETSIZE];
        let mut cb = Cb::default();
        let cb = &mut cb;
        println!("");

        let mut client = Connection::new();
        let mut server = Connection::new();

        // Token request
        assert!(cb.random_value.replace([0x12, 0x34, 0x56, 0x78]).is_none());
        client.connect(cb).void_unwrap();
        let packet = cb.sent.pop_front().unwrap();
        assert!(cb.sent.is_empty());
        hexdump(&packet);
        let expected = {
            let mut expected = Vec::new();
            expected.extend_from_slice(b"\x04\x00\x00\xff\xff\xff\xff\x05\x12\x34\x56\x78");
            expected.extend_from_slice(&[0; protocol::TOKEN_REQUEST_PACKET_SIZE - 12]);
            expected
        };
        // FIXME(rust-lang/rust#87555): Use concat_bytes!
        assert!(&packet == &expected);

        // Token response
        assert!(cb.random_value.replace([0x9a, 0xbc, 0xde, 0xf0]).is_none());
        assert!(server
            .feed(cb, &mut Panic, &packet, &mut buffer[..])
            .0
            .next()
            .is_none());
        let packet = cb.sent.pop_front().unwrap();
        assert!(cb.sent.is_empty());
        hexdump(&packet);
        assert!(&packet == b"\x04\x00\x00\x12\x34\x56\x78\x05\x9a\xbc\xde\xf0");

        // Connect
        assert!(client
            .feed(cb, &mut Panic, &packet, &mut buffer[..])
            .0
            .next()
            .is_none());
        let packet = cb.sent.pop_front().unwrap();
        assert!(cb.sent.is_empty());
        hexdump(&packet);
        assert!(&packet == b"\x04\x00\x00\x9a\xbc\xde\xf0\x01\x12\x34\x56\x78");

        // Accept
        assert!(server
            .feed(cb, &mut Panic, &packet, &mut buffer[..])
            .0
            .next()
            .is_none());
        let packet = cb.sent.pop_front().unwrap();
        assert!(cb.sent.is_empty());
        hexdump(&packet);
        assert!(&packet == b"\x04\x00\x00\x12\x34\x56\x78\x02");

        assert!(
            client
                .feed(cb, &mut Panic, &packet, &mut buffer[..])
                .0
                .collect_vec()
                == &[ReceiveChunk::Ready]
        );
        assert!(cb.sent.is_empty());

        // Send
        client.send(cb, b"\x42", true).unwrap();
        assert!(cb.sent.is_empty());

        // Flush
        client.flush(cb).void_unwrap();
        let packet = cb.sent.pop_front().unwrap();
        assert!(cb.sent.is_empty());
        hexdump(&packet);
        assert!(&packet == b"\x00\x00\x01\x9a\xbc\xde\xf0\x40\x01\x01\x42");

        // Receive
        assert!(
            server
                .feed(cb, &mut Panic, &packet, &mut buffer[..])
                .0
                .collect_vec()
                == &[ReceiveChunk::Connected(b"\x42", true)]
        );
        assert!(cb.sent.is_empty());

        // Disconnect
        server.disconnect(cb, b"42").void_unwrap();
        let packet = cb.sent.pop_front().unwrap();
        hexdump(&packet);
        assert!(&packet == b"\x04\x01\x00\x12\x34\x56\x78\x0442\x00");

        assert!(
            client
                .feed(cb, &mut Panic, &packet, &mut buffer[..])
                .0
                .collect_vec()
                == &[ReceiveChunk::Disconnect(b"42")]
        );

        client.reset();
        server.reset();
    }
}
