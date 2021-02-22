use Connection;
use Timeout;
use Timestamp;
use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use connection::ReceiveChunk;
use connection;
use collections::PeerMap;
use collections::peer_map;
use protocol::ConnectedPacket;
use protocol::ConnectedPacketType;
use protocol::ControlPacket;
use protocol::Packet;
use protocol;
use std::fmt;
use std::hash::Hash;
use std::iter;
use std::ops;
use warn::Panic;
use warn::Warn;

pub use connection::Error;

pub trait Callback<A: Address> {
    type Error;
    fn send(&mut self, addr: A, data: &[u8]) -> Result<(), Self::Error>;
    fn time(&mut self) -> Timestamp;
}

#[derive(Debug)]
pub enum Warning<A: Address> {
    Peer(A, PeerId, connection::Warning),
    Connless(A, connection::Warning),
}

impl<A: Address> Warning<A> {
    pub fn addr(&self) -> A {
        match *self {
            Warning::Peer(addr, _, _) => addr,
            Warning::Connless(addr, _) => addr,
        }
    }
}

pub trait Address: Copy + Eq + Hash + Ord { }
impl<A: Copy + Eq + Hash + Ord> Address for A { }

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PeerId(pub u32);

impl PeerId {
    fn get_and_increment(&mut self) -> PeerId {
        let old = *self;
        self.0 = self.0.wrapping_add(1);
        old
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p{}", self.0)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

const CONNECT_PACKET: &'static [u8; 4] = b"\x10\x00\x00\x01";

struct Peer<A: Address> {
    conn: Connection,
    addr: A,
}

impl<A: Address> Peer<A> {
    fn new(addr: A) -> Peer<A> {
        Peer {
            conn: Connection::new(),
            addr: addr,
        }
    }
}

struct Peers<A: Address> {
    peers: PeerMap<Peer<A>>,
    next_peer_id: PeerId,
}

impl<A: Address> Peers<A> {
    fn new() -> Peers<A> {
        Peers {
            peers: PeerMap::new(),
            next_peer_id: PeerId(0),
        }
    }
    fn new_peer(&mut self, addr: A) -> (PeerId, &mut Peer<A>) {
        // FIXME(rust-lang/rfcs#811): Work around missing non-lexical borrows.
        let raw_self: *mut Peers<A> = self;
        unsafe {
            loop {
                let peer_id = self.next_peer_id.get_and_increment();
                if let peer_map::Entry::Vacant(v) = (*raw_self).peers.entry(peer_id) {
                    return (peer_id, v.insert(Peer::new(addr)));
                }
            }
        }
    }
    fn iter(&self) -> peer_map::Iter<Peer<A>> {
        self.peers.iter()
    }
    fn iter_mut(&mut self) -> peer_map::IterMut<Peer<A>> {
        self.peers.iter_mut()
    }
    fn remove_peer(&mut self, pid: PeerId) {
        self.peers.remove(pid)
    }
    fn pid_from_addr(&mut self, addr: A) -> Option<PeerId> {
        for (pid, p) in self.peers.iter() {
            if p.addr == addr {
                return Some(pid);
            }
        }
        None
    }
    fn get(&self, pid: PeerId) -> Option<&Peer<A>> {
        self.peers.get(pid)
    }
    fn get_mut(&mut self, pid: PeerId) -> Option<&mut Peer<A>> {
        self.peers.get_mut(pid)
    }
}

impl<A: Address> ops::Index<PeerId> for Peers<A> {
    type Output = Peer<A>;
    fn index(&self, pid: PeerId) -> &Peer<A> {
        self.get(pid).unwrap_or_else(|| panic!("invalid pid"))
    }
}

impl<A: Address> ops::IndexMut<PeerId> for Peers<A> {
    fn index_mut(&mut self, pid: PeerId) -> &mut Peer<A> {
        self.get_mut(pid).unwrap_or_else(|| panic!("invalid pid"))
    }
}

// TODO: Simplify these enums. A lot.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkOrEvent<'a, A: Address> {
    Chunk(Chunk<'a>),
    Connless(ConnlessChunk<'a, A>),
    Connect(PeerId),
    Ready(PeerId),
    Disconnect(PeerId, &'a [u8]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chunk<'a> {
    pub pid: PeerId,
    pub vital: bool,
    pub data: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnlessChunk<'a, A: Address> {
    pub addr: A,
    pub pid: Option<PeerId>,
    pub data: &'a [u8],
}

struct ConnlessBuilder {
    buffer: [u8; protocol::MAX_PACKETSIZE],
}

impl ConnlessBuilder {
    fn new() -> ConnlessBuilder {
        ConnlessBuilder {
            buffer: [0; protocol::MAX_PACKETSIZE],
        }
    }
    fn send<A: Address, CB: Callback<A>>(&mut self, cb: &mut CB, addr: A, packet: Packet)
        -> Result<(), Error<CB::Error>>
    {
        let send_data = match packet.write(&mut [0u8; 0][..], &mut self.buffer[..]) {
            Ok(d) => d,
            Err(protocol::Error::Capacity(_)) => unreachable!("too short buffer provided"),
            Err(protocol::Error::TooLongData) => return Err(Error::TooLongData),
        };
        cb.send(addr, send_data)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ReceivePacket<'a, A: Address> {
    type_: ReceivePacketType<'a, A>,
}

impl<'a, A: Address> Iterator for ReceivePacket<'a, A> {
    type Item = ChunkOrEvent<'a, A>;
    fn next(&mut self) -> Option<ChunkOrEvent<'a, A>> {
        use self::ReceivePacketType::Connect;
        use self::ReceivePacketType::Connected;
        use self::ReceivePacketType::Connless;
        match self.type_ {
            ReceivePacketType::None => None,
            Connect(ref mut once) => once.next().map(|pid| ChunkOrEvent::Connect(pid)),
            Connected(addr, pid, ref mut receive_packet) => receive_packet.next().map(|chunk| {
                match chunk {
                    ReceiveChunk::Connless(d) => ChunkOrEvent::Connless(ConnlessChunk {
                        addr: addr,
                        pid: Some(pid),
                        data: d,
                    }),
                    ReceiveChunk::Connected(d, vital) => ChunkOrEvent::Chunk(Chunk {
                        pid: pid,
                        vital: vital,
                        data: d,
                    }),
                    ReceiveChunk::Ready => ChunkOrEvent::Ready(pid),
                    ReceiveChunk::Disconnect(r) => ChunkOrEvent::Disconnect(pid, r),
                }
            }),
            Connless(addr, ref mut once) => once.next().map(|data| {
                ChunkOrEvent::Connless(ConnlessChunk {
                    addr: addr,
                    pid: None,
                    data: data,
                })
            }),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.clone().count();
        (len, Some(len))
    }
}

impl<'a, A: Address> ExactSizeIterator for ReceivePacket<'a, A> { }

impl<'a, A: Address> ReceivePacket<'a, A> {
    fn none() -> ReceivePacket<'a, A> {
        ReceivePacket {
            type_: ReceivePacketType::None,
        }
    }
    fn connect(pid: PeerId) -> ReceivePacket<'a, A> {
        ReceivePacket {
            type_: ReceivePacketType::Connect(iter::once(pid)),
        }
    }
    fn connected(addr: A, pid: PeerId, receive_packet: connection::ReceivePacket<'a>, net: &mut Net<A>)
        -> ReceivePacket<'a, A>
    {
        for chunk in receive_packet.clone() {
            if let ReceiveChunk::Disconnect(..) = chunk {
                net.peers.remove_peer(pid);
            }
        }
        ReceivePacket {
            type_: ReceivePacketType::Connected(addr, pid, receive_packet),
        }
    }

    fn connless(addr: A, data: &'a [u8]) -> ReceivePacket<'a, A> {
        ReceivePacket {
            type_: ReceivePacketType::Connless(addr, iter::once(data)),
        }
    }
}

#[derive(Clone)]
enum ReceivePacketType<'a, A: Address> {
    None,
    Connect(iter::Once<PeerId>),
    Connected(A, PeerId, connection::ReceivePacket<'a>),
    Connless(A, iter::Once<&'a [u8]>),
}

pub struct Net<A: Address> {
    peers: Peers<A>,
    builder: ConnlessBuilder,
    accept_connections: bool,
}

struct ConnectionCallback<'a, A: Address, CB: Callback<A>+'a> {
    cb: &'a mut CB,
    addr: A,
}

// Create `ConnectionCallback`.
fn cc<A: Address, CB: Callback<A>>(cb: &mut CB, addr: A) -> ConnectionCallback<A, CB> {
    ConnectionCallback {
        cb: cb,
        addr: addr,
    }
}

impl<'a, A: Address, W: Warn<Warning<A>>> Warn<connection::Warning> for WarnCallback<'a, A, W> {
    fn warn(&mut self, warning: connection::Warning) {
        self.warn.warn(Warning::Connless(self.addr, warning))
    }
}

impl<'a, A: Address, W: Warn<Warning<A>>> Warn<protocol::Warning> for WarnCallback<'a, A, W> {
    fn warn(&mut self, warning: protocol::Warning) {
        self.warn.warn(Warning::Connless(self.addr, connection::Warning::Packet(warning)))
    }
}

struct WarnCallback<'a, A: Address, W: Warn<Warning<A>>+'a> {
    warn: &'a mut W,
    addr: A,
}

fn w<A: Address, W: Warn<Warning<A>>>(warn: &mut W, addr: A) -> WarnCallback<A, W> {
    WarnCallback {
        warn: warn,
        addr: addr,
    }
}

impl<'a, A: Address, W: Warn<Warning<A>>> Warn<connection::Warning> for WarnPeerCallback<'a, A, W> {
    fn warn(&mut self, warning: connection::Warning) {
        self.warn.warn(Warning::Peer(self.addr, self.pid, warning))
    }
}

struct WarnPeerCallback<'a, A: Address, W: Warn<Warning<A>>+'a> {
    warn: &'a mut W,
    addr: A,
    pid: PeerId,
}

fn wp<A: Address, W: Warn<Warning<A>>>(warn: &mut W, addr: A, pid: PeerId)
    -> WarnPeerCallback<A, W>
{
    WarnPeerCallback {
        warn: warn,
        addr: addr,
        pid: pid,
    }
}

impl<'a, A: Address, CB: Callback<A>> connection::Callback for ConnectionCallback<'a, A, CB> {
    type Error = CB::Error;
    fn send(&mut self, data: &[u8]) -> Result<(), CB::Error> {
        self.cb.send(self.addr, data)
    }
    fn time(&mut self) -> Timestamp {
        self.cb.time()
    }
}

impl<A: Address> Net<A> {
    fn new(accept_connections: bool) -> Net<A> {
        Net {
            peers: Peers::new(),
            builder: ConnlessBuilder::new(),
            accept_connections: accept_connections,
        }
    }
    pub fn server() -> Net<A> {
        Net::new(true)
    }
    pub fn client() -> Net<A> {
        Net::new(false)
    }
    pub fn needs_tick(&self) -> Timeout {
        self.peers.iter().map(|(_, p)| p.conn.needs_tick()).min().unwrap_or_default()
    }
    pub fn is_receive_chunk_still_valid(&self, chunk: &mut ChunkOrEvent<A>) -> bool {
        if let ChunkOrEvent::Chunk(Chunk { pid, .. }) = *chunk {
            self.peers.get(pid).is_some()
        } else {
            true
        }
    }
    pub fn connect<CB: Callback<A>>(&mut self, cb: &mut CB, addr: A)
        -> (PeerId, Result<(), CB::Error>)
    {
        let (pid, peer) = self.peers.new_peer(addr);
        (pid, peer.conn.connect(&mut cc(cb, peer.addr)))
    }
    pub fn disconnect<CB: Callback<A>>(&mut self, cb: &mut CB, pid: PeerId, reason: &[u8])
        -> Result<(), CB::Error>
    {
        let result;
        {
            let peer = &mut self.peers[pid];
            assert!(!peer.conn.is_unconnected());
            result = peer.conn.disconnect(&mut cc(cb, peer.addr), reason);
        }
        self.peers.remove_peer(pid);
        result
    }
    pub fn send_connless<CB: Callback<A>>(&mut self, cb: &mut CB, addr: A, data: &[u8])
        -> Result<(), Error<CB::Error>>
    {
        self.builder.send(cb, addr, Packet::Connless(data))
    }
    pub fn send<CB: Callback<A>>(&mut self, cb: &mut CB, chunk: Chunk)
        -> Result<(), Error<CB::Error>>
    {
        let peer = &mut self.peers[chunk.pid];
        peer.conn.send(&mut cc(cb, peer.addr), chunk.data, chunk.vital)
    }
    pub fn flush<CB: Callback<A>>(&mut self, cb: &mut CB, pid: PeerId)
        -> Result<(), CB::Error>
    {
        let peer = &mut self.peers[pid];
        peer.conn.flush(&mut cc(cb, peer.addr))
    }
    pub fn ignore(&mut self, pid: PeerId) {
        self.peers.remove_peer(pid);
    }
    pub fn accept<CB: Callback<A>>(&mut self, cb: &mut CB, pid: PeerId)
        -> Result<(), CB::Error>
    {
        let peer = &mut self.peers[pid];
        assert!(peer.conn.is_unconnected());
        let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
        let (mut none, res) =
            peer.conn.feed(&mut cc(cb, peer.addr), &mut Panic, CONNECT_PACKET, &mut buf);
        assert!(none.next().is_none());
        res
    }
    pub fn reject<CB: Callback<A>>(&mut self, cb: &mut CB, pid: PeerId, reason: &[u8])
        -> Result<(), CB::Error>
    {
        let result;
        {
            let peer = &mut self.peers[pid];
            assert!(peer.conn.is_unconnected());
            result = peer.conn.disconnect(&mut cc(cb, peer.addr), reason);
        }
        self.peers.remove_peer(pid);
        result
    }
    pub fn tick<'a, CB: Callback<A>>(&'a mut self, cb: &'a mut CB)
        -> Tick<A, CB>
    {
        Tick {
            iter_mut: self.peers.iter_mut(),
            cb: cb,
        }
    }
    pub fn feed<'a, CB, B, W>(&mut self, cb: &mut CB, warn: &mut W, addr: A, data: &'a [u8], buf: B)
        -> (ReceivePacket<'a, A>, Result<(), CB::Error>)
        where CB: Callback<A>,
              B: Buffer<'a>,
              W: Warn<Warning<A>>,
    {
        with_buffer(buf, |b| self.feed_impl(cb, warn, addr, data, b))
    }
    fn feed_impl<'d, 's, CB, W>(&mut self, cb: &mut CB, warn: &mut W, addr: A, data: &'d [u8], mut buf: BufferRef<'d, 's>)
        -> (ReceivePacket<'d, A>, Result<(), CB::Error>)
        where CB: Callback<A>,
              W: Warn<Warning<A>>,
    {
        if let Some(pid) = self.peers.pid_from_addr(addr) {
            let (packet, e) = self.peers[pid].conn.feed(&mut cc(cb, addr), &mut wp(warn, addr, pid), data, &mut buf);
            (ReceivePacket::connected(addr, pid, packet, self), e)
        } else {
            let packet = match Packet::read(&mut w(warn, addr), data, &mut buf) {
                Ok(p) => p,
                Err(e) => {
                    w(warn, addr).warn(connection::Warning::Read(e));
                    return (ReceivePacket::none(), Ok(()));
                }
            };
            if let Packet::Connless(d) = packet {
                (ReceivePacket::connless(addr, d), Ok(()))
            } else if let Packet::Connected(ConnectedPacket {
                    type_: ConnectedPacketType::Control(ControlPacket::Connect), ..
                }) = packet
            {
                if self.accept_connections {
                    let (pid, _) = self.peers.new_peer(addr);
                    (ReceivePacket::connect(pid), Ok(()))
                } else {
                    w(warn, addr).warn(connection::Warning::Unexpected);
                    (ReceivePacket::none(), Ok(()))
                }
            } else {
                w(warn, addr).warn(connection::Warning::Unexpected);
                (ReceivePacket::none(), Ok(()))
            }
        }
    }
}

pub struct Tick<'a, A: Address+'a, CB: Callback<A>+'a> {
    iter_mut: peer_map::IterMut<'a, Peer<A>>,
    cb: &'a mut CB,
}

impl<'a, A: Address+'a, CB: Callback<A>+'a> Iterator for Tick<'a, A, CB> {
    type Item = CB::Error;
    fn next(&mut self) -> Option<CB::Error> {
        while let Some((_, p)) = self.iter_mut.next() {
            match p.conn.tick(&mut cc(self.cb, p.addr)) {
                Ok(()) => {},
                Err(e) => return Some(e),
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use Timestamp;
    use itertools::Itertools;
    use protocol;
    use std::collections::VecDeque;
    use super::Callback;
    use super::ChunkOrEvent;
    use super::Net;
    use void::ResultVoidExt;
    use void::Void;
    use warn::Panic;

    #[test]
    fn establish_connection() {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        enum Address {
            Client,
            Server,
        }
        struct Cb {
            packets: VecDeque<Vec<u8>>,
            recipient: Address,
        }
        impl Cb {
            fn new() -> Cb {
                Cb {
                    packets: VecDeque::new(),
                    recipient: Address::Server,
                }
            }
        }
        impl Callback<Address> for Cb {
            type Error = Void;
            fn send(&mut self, addr: Address, data: &[u8]) -> Result<(), Void> {
                assert!(self.recipient == addr);
                self.packets.push_back(data.to_owned());
                Ok(())
            }
            fn time(&mut self) -> Timestamp {
                Timestamp::from_secs_since_epoch(0)
            }
        }
        let mut cb = Cb::new();
        let cb = &mut cb;
        let mut buffer = [0; protocol::MAX_PACKETSIZE];

        let mut net = Net::server();

        // Connect
        cb.recipient = Address::Server;
        let (c_pid, res) = net.connect(cb, Address::Server);
        res.void_unwrap();
        let packet = cb.packets.pop_front().unwrap();
        assert!(cb.packets.is_empty());

        // ConnectAccept
        cb.recipient = Address::Client;
        let s_pid;
        {
            let p = net.feed(cb, &mut Panic, Address::Client, &packet, &mut buffer[..]).0.collect_vec();
            assert!(p.len() == 1);
            if let ChunkOrEvent::Connect(s) = p[0] {
                s_pid = s;
            } else {
                panic!();
            }
        }
        // No packets sent out until we accept the client.
        assert!(cb.packets.is_empty());

        net.accept(cb, s_pid).void_unwrap();
        let packet = cb.packets.pop_front().unwrap();
        assert!(cb.packets.is_empty());

        // Accept
        cb.recipient = Address::Server;
        assert!(net.feed(cb, &mut Panic, Address::Server, &packet, &mut buffer[..]).0.collect_vec()
                == &[ChunkOrEvent::Ready(c_pid)]);
        let packet = cb.packets.pop_front().unwrap();
        assert!(cb.packets.is_empty());

        cb.recipient = Address::Client;
        assert!(net.feed(cb, &mut Panic, Address::Client, &packet, &mut buffer[..]).0.next().is_none());
        assert!(cb.packets.is_empty());

        // Disconnect
        cb.recipient = Address::Server;
        net.disconnect(cb, c_pid, b"foobar").void_unwrap();
        let packet = cb.packets.pop_front().unwrap();
        assert!(cb.packets.is_empty());

        cb.recipient = Address::Client;
        assert!(net.feed(cb, &mut Panic, Address::Client, &packet, &mut buffer[..]).0.collect_vec()
                == &[ChunkOrEvent::Disconnect(s_pid, b"foobar")]);
        assert!(cb.packets.is_empty());
    }
}
