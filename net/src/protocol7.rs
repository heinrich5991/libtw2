use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use common::num::Cast;
use common::pretty;
use huffman::instances::TEEWORLDS as HUFFMAN;
use huffman;
use std::cmp;
use std::fmt;
use warn::Ignore;
use warn::Warn;

pub const CHUNK_HEADER_SIZE: usize = 2;
pub const CHUNK_HEADER_SIZE_VITAL: usize = 3;
pub const HEADER_SIZE: usize = 7;
pub const HEADER_SIZE_CONNLESS: usize = 9;
pub const MAX_PACKETSIZE: usize = 1400;

// For connectionless packets, this is obvious (MAX_PACKETSIZE -
// HEADER_SIZE_CONNLESS). For packets sent in a connection context, you also get
// a chunk header which, which makes the maximum payload size (MAX_PACKETSIZE -
// HEADER_SIZE - CHUNK_HEADER_SIZE_VITAL).
pub const MAX_PAYLOAD: usize = 1390;

pub const PACKETFLAG_CONTROL:        u8 = 1 << 0;
pub const PACKETFLAG_REQUEST_RESEND: u8 = 1 << 1;
pub const PACKETFLAG_COMPRESSION:    u8 = 1 << 2;
pub const PACKETFLAG_CONNLESS:       u8 = 1 << 3;

pub const CHUNKFLAG_VITAL:  u8 = 1 << 0;
pub const CHUNKFLAG_RESEND: u8 = 1 << 1;

pub const CTRLMSG_KEEPALIVE:     u8 = 0;
pub const CTRLMSG_CONNECT:       u8 = 1;
pub const CTRLMSG_ACCEPT:        u8 = 2;
pub const CTRLMSG_CLOSE:         u8 = 4;
pub const CTRLMSG_TOKEN:         u8 = 5;

pub const CONNLESS_VERSION: u8 = 1;
pub const CTRLMSG_CLOSE_REASON_LENGTH: usize = 127;
pub const TOKEN_REQUEST_PACKET_SIZE: usize = 519;
pub const TOKEN_NONE: Token = Token([0xff, 0xff, 0xff, 0xff]);

pub const CHUNK_FLAGS_BITS: u32 = 2;
pub const CHUNK_SIZE_BITS: u32 = 12;
pub const PACKET_FLAGS_BITS: u32 = 4;
pub const SEQUENCE_BITS: u32 = 10;
pub const SEQUENCE_MODULUS: u16 = 1 << SEQUENCE_BITS;
pub const VERSION_BITS: u32 = 2;

pub fn chunk_header_size(vital: bool) -> usize {
    if vital {
        CHUNK_HEADER_SIZE_VITAL
    } else {
        CHUNK_HEADER_SIZE
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Warning {
    ChunkHeaderPadding,
    ChunksNoChunks,
    ChunksNumChunks,
    ChunksUnknownData,
    ConnlessFlags,
    ControlExcessData,
    ControlFlags,
    ControlNulTermination,
    ControlNumChunks,
    PacketHeaderPadding,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PacketReadError {
    Compression,
    ControlMissing,
    ControlResponseTokenMissing,
    ControlTokenRequestTooShort,
    TooLong,
    TooShort,
    UnknownConnlessVersion,
    UnknownControl,
}

#[derive(Clone, Copy)]
pub enum ControlPacket<'a> {
    KeepAlive,
    /// Connect(response_token)
    Connect(Token),
    Accept,
    /// Close(reason)
    Close(&'a [u8]),
    /// Token(response_token)
    Token(Token),
}

impl<'a> fmt::Debug for ControlPacket<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ControlPacket::KeepAlive => f.debug_tuple("KeepAlive").finish(),
            ControlPacket::Connect(rt) => f.debug_tuple("Connect").field(rt).finish(),
            ControlPacket::Accept => f.debug_tuple("Accept").finish(),
            ControlPacket::Close(reason) =>
                f.debug_tuple("Close").field(&pretty::AlmostString::new(reason)).finish(),
            ControlPacket::Token(rt) => f.debug_tuple("Token").field(rt).finish(),
        }
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Token(pub [u8; 4]);

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08x}", u32::from_be_bytes(self.0))
    }
}

impl<'a> Packet<'a> {
    fn needs_decompression(packet: &[u8]) -> bool {
        if packet.len() > MAX_PACKETSIZE {
            return false;
        }
        let (header, _) = unwrap_or_return!(
            PacketHeaderPacked::from_byte_slice(packet),
            false
        );
        let header = header.unpack_warn(&mut Ignore);
        header.flags & PACKETFLAG_CONNLESS == 0 &&
            header.flags & PACKETFLAG_COMPRESSION != 0
    }
    /// Parse a packet.
    ///
    /// `buffer` needs to have at least size `MAX_PAYLOAD`.
    pub fn read<'b, B, W>(warn: &mut W, bytes: &'b [u8], buffer: B)
        -> Result<Packet<'b>, PacketReadError>
        where B: Buffer<'b>,
              W: Warn<Warning>,
    {
        with_buffer(buffer, |b| Packet::read_impl(warn, bytes, Some(b)))
    }
    pub fn read_panic_on_decompression<'b, W>(warn: &mut W, bytes: &'b [u8])
        -> Result<Packet<'b>, PacketReadError>
        where W: Warn<Warning>,
    {
        Packet::read_impl(warn, bytes, None)
    }
    fn read_impl<'d, 's, W>(
        warn: &mut W,
        bytes: &'d [u8],
        buffer: Option<BufferRef<'d, 's>>,
    ) -> Result<Packet<'d>, PacketReadError>
        where W: Warn<Warning>,
    {
        use self::PacketReadError::*;

        assert!(buffer.as_ref().map(|b| b.remaining() >= MAX_PACKETSIZE).unwrap_or(true));
        if bytes.len() > MAX_PACKETSIZE {
            return Err(TooLong);
        }
        let (header, payload) = unwrap_or_return!(PacketHeaderPacked::from_byte_slice(bytes), Err(TooShort));
        let header = header.unpack_warn(warn);
        if header.flags & PACKETFLAG_CONNLESS != 0 {
            let (header, payload) = unwrap_or_return!(PacketHeaderConnlessPacked::from_byte_slice(bytes), Err(TooShort));
            let header = header.unpack_warn(warn);
            if header.version != CONNLESS_VERSION {
                return Err(UnknownConnlessVersion);
            }
            if header.flags & PACKETFLAG_COMPRESSION != 0
                || header.flags & PACKETFLAG_REQUEST_RESEND != 0
                || header.flags & PACKETFLAG_CONTROL != 0
            {
                // TODO: Should we handle these flags? Vanilla does that too.
                warn.warn(Warning::ConnlessFlags);
            }

            return Ok(Packet::Connless(payload));
        }

        let payload = if header.flags & PACKETFLAG_COMPRESSION != 0 {
            let mut buffer = buffer.expect("read_panic_on_decompression called on compressed packet");
            let decompressed = Packet::decompress(bytes, &mut buffer)
                .map_err(|_| Compression)?;
            let (_, payload) = PacketHeaderPacked::from_byte_slice(decompressed)
                .unwrap();
            payload
        } else {
            payload
        };

        if payload.len() > MAX_PAYLOAD {
            return Err(Compression);
        }

        let ack = header.ack;
        let type_ = if header.flags & PACKETFLAG_CONTROL != 0 {
            if header.num_chunks != 0 {
                warn.warn(Warning::ControlNumChunks);
            }
            if header.flags & PACKETFLAG_COMPRESSION != 0
                || header.flags & PACKETFLAG_REQUEST_RESEND != 0
            {
                // TODO: Should we handle these flags? Vanilla does that too.
                warn.warn(Warning::ControlFlags);
            }

            let (&control, payload) = unwrap_or_return!(payload.split_first(),
                                                        Err(ControlMissing));
            let empty = |warn: &mut W| {
                if !payload.is_empty() {
                    warn.warn(Warning::ControlExcessData);
                }
            };
            let token = |warn: &mut W, warn_more: bool| {
                if payload.len() < 4 {
                    return Err(ControlResponseTokenMissing);
                }
                let (token, rest) = payload.split_at(4);
                if warn_more && !rest.is_empty() {
                    warn.warn(Warning::ControlExcessData);
                }
                Ok(Token([token[0], token[1], token[2], token[3]]))
            };
            let control = match control {
                CTRLMSG_KEEPALIVE => {
                    empty(warn);
                    ControlPacket::KeepAlive
                },
                CTRLMSG_CONNECT => {
                    ControlPacket::Connect(token(warn, true)?)
                },
                CTRLMSG_ACCEPT => {
                    empty(warn);
                    ControlPacket::Accept
                },
                CTRLMSG_CLOSE => {
                    let nul = payload.iter().position(|&b| b == 0).unwrap_or(payload.len());
                    let nul = cmp::min(nul, CTRLMSG_CLOSE_REASON_LENGTH);
                    if payload.len() != 0 && nul + 1 != payload.len() {
                        if nul + 1 < payload.len() {
                            warn.warn(Warning::ControlExcessData);
                        } else {
                            warn.warn(Warning::ControlNulTermination);
                        }
                    }
                    ControlPacket::Close(&payload[..nul])
                },
                CTRLMSG_TOKEN => {
                    if header.token == TOKEN_NONE && bytes.len() < TOKEN_REQUEST_PACKET_SIZE {
                        return Err(ControlTokenRequestTooShort)
                    }
                    ControlPacket::Token(token(warn, header.token != TOKEN_NONE)?)
                },
                _ => {
                    // Unrecognized control packet.
                    return Err(UnknownControl);
                },
            };

            ConnectedPacketType::Control(control)
        } else {
            let request_resend = header.flags & PACKETFLAG_REQUEST_RESEND != 0;
            if header.num_chunks == 0 && !request_resend {
                warn.warn(Warning::ChunksNoChunks);
            }
            ConnectedPacketType::Chunks(request_resend, header.num_chunks, payload)
        };

        Ok(Packet::Connected(ConnectedPacket {
            ack: ack,
            type_: type_,
        }))
    }
    /// `buffer` needs to have at least size `MAX_PACKETSIZE`.
    pub fn decompress_if_needed<B: Buffer<'a>>(packet: &[u8], buffer: B)
        -> Result<bool, huffman::DecompressionError>
    {
        with_buffer(buffer, |b| Packet::decompress_if_needed_impl(packet, b))
    }
    fn decompress_if_needed_impl<'d, 's>(
        packet: &[u8],
        mut buffer: BufferRef<'d, 's>,
    ) -> Result<bool, huffman::DecompressionError> {
        assert!(buffer.remaining() >= MAX_PACKETSIZE);
        if !Packet::needs_decompression(packet) {
            return Ok(false);
        }
        Packet::decompress(packet, &mut buffer)?;
        Ok(true)
    }

    fn decompress<B: Buffer<'a>>(packet: &[u8], buffer: B)
        -> Result<&'a [u8], huffman::DecompressionError>
    {
        with_buffer(buffer, |b| Packet::decompress_impl(packet, b))
    }
    fn decompress_impl<'d, 's>(packet: &[u8], mut buffer: BufferRef<'d, 's>)
        -> Result<&'d [u8], huffman::DecompressionError>
    {
        assert!(buffer.remaining() >= MAX_PACKETSIZE);
        assert!(Packet::needs_decompression(packet));
        let (header, payload) = PacketHeaderPacked::from_byte_slice(packet)
            .expect("packet passed to decompress too short for header");
        let header = header.unpack_warn(&mut Ignore);
        assert!(header.flags & PACKETFLAG_CONNLESS == 0);
        assert!(header.flags & PACKETFLAG_COMPRESSION != 0);

        let fake_header = PacketHeader {
            flags: header.flags & !PACKETFLAG_COMPRESSION,
            ack: header.ack,
            num_chunks: header.num_chunks,
            token: header.token,
        };
        buffer.write(fake_header.pack().as_bytes()).unwrap();
        HUFFMAN.decompress(payload, &mut buffer)?;

        Ok(buffer.initialized())
    }
}


#[derive(Clone, Copy, Debug)]
pub struct ConnectedPacket<'a> {
    pub ack: u16, // u10
    pub type_: ConnectedPacketType<'a>,
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectedPacketType<'a> {
    // Chunks(request_resend, num_chunks, payload)
    Chunks(bool, u8, &'a [u8]),
    Control(ControlPacket<'a>),
}

#[derive(Clone, Copy, Debug)]
pub enum Packet<'a> {
    Connless(&'a [u8]),
    Connected(ConnectedPacket<'a>),
}

#[derive(Clone, Copy, Debug)]
pub struct Chunk<'a> {
    pub data: &'a [u8],
    // vital: Some((sequence, resend))
    pub vital: Option<(u16, bool)>,
}

#[derive(Clone, Debug)]
pub struct ChunksIter<'a> {
    data: &'a [u8],
    initial_len: usize,
    num_remaining_chunks: i32,
    checked_num_chunks_warning: bool,
}

impl<'a> ChunksIter<'a> {
    pub fn new(data: &'a [u8], num_chunks: u8) -> ChunksIter<'a> {
        ChunksIter {
            data: data,
            initial_len: data.len(),
            num_remaining_chunks: num_chunks.i32(),
            checked_num_chunks_warning: false,
        }
    }
    fn excess_data<W: Warn<Warning>>(&mut self, warn: &mut W) -> Option<Chunk<'static>> {
        warn.warn(Warning::ChunksUnknownData);
        self.data = &[];
        None
    }
    pub fn pos(&self) -> usize {
        self.initial_len - self.data.len()
    }
    pub fn next_warn<W>(&mut self, warn: &mut W) -> Option<Chunk<'a>>
        where W: Warn<Warning>
    {
        if self.data.len() == 0 {
            if !self.checked_num_chunks_warning {
                self.checked_num_chunks_warning = true;
                if self.num_remaining_chunks != 0 {
                    warn.warn(Warning::ChunksNumChunks);
                }
            }
            return None;
        }
        let (header, sequence, chunk_data_and_rest) = unwrap_or_return!(
            read_chunk_header(warn, self.data),
            self.excess_data(warn)
        );
        let vital = sequence.map(|s| (s, header.flags & CHUNKFLAG_RESEND != 0));
        let size = header.size.usize();
        if chunk_data_and_rest.len() < size {
            return self.excess_data(warn);
        }
        let (chunk_data, rest) = chunk_data_and_rest.split_at(size);
        self.data = rest;
        self.num_remaining_chunks -= 1;
        Some(Chunk {
            data: chunk_data,
            vital: vital,
        })
    }
}

impl<'a> Iterator for ChunksIter<'a> {
    type Item = Chunk<'a>;
    fn next(&mut self) -> Option<Chunk<'a>> {
        self.next_warn(&mut Ignore)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.clone().count();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for ChunksIter<'a> { }

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PacketHeaderPacked {
    padding_flags_ack: u8, // u2 u4 u2
    ack: u8,
    num_chunks: u8,
    token: [u8; 4],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PacketHeader {
    pub flags: u8, // u4
    pub ack: u16, // u10
    pub num_chunks: u8,
    pub token: Token,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PacketHeaderConnlessPacked {
    padding_flags_version: u8, // u2 u4 u2
    token: [u8; 4],
    response_token: [u8; 4],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PacketHeaderConnless {
    pub flags: u8, // u4
    pub version: u8, // u2
    pub token: Token,
    pub response_token: Token,
}

impl PacketHeaderPacked {
    pub fn unpack_warn<W: Warn<Warning>>(self, warn: &mut W) -> PacketHeader {
        let PacketHeaderPacked { padding_flags_ack, ack, num_chunks, token } = self;
        if padding_flags_ack & 0b1100_0000 != 0 {
            warn.warn(Warning::PacketHeaderPadding);
        }
        PacketHeader {
            flags: (padding_flags_ack & 0b0011_1100) >> 2,
            ack: (((padding_flags_ack & 0b0000_0011) as u16) << 8) | (ack as u16),
            num_chunks,
            token: Token(token),
        }
    }
    pub fn unpack(self) -> PacketHeader {
        self.unpack_warn(&mut Ignore)
    }
}

impl PacketHeader {
    pub fn pack(self) -> PacketHeaderPacked {
        let PacketHeader { flags, ack, num_chunks, token } = self;
        // Check that the fields do not exceed their maximal size.
        assert!(flags >> PACKET_FLAGS_BITS == 0);
        assert!(ack >> SEQUENCE_BITS == 0);
        PacketHeaderPacked {
            padding_flags_ack: flags << 2 | (ack >> 8) as u8,
            ack: ack as u8,
            num_chunks,
            token: token.0,
        }
    }
}

impl PacketHeaderConnlessPacked {
    pub fn unpack_warn<W: Warn<Warning>>(self, warn: &mut W) -> PacketHeaderConnless {
        let PacketHeaderConnlessPacked { padding_flags_version, token, response_token } = self;
        if padding_flags_version & 0b1100_0000 != 0 {
            warn.warn(Warning::PacketHeaderPadding);
        }
        PacketHeaderConnless {
            flags: (padding_flags_version & 0b0011_1100) >> 2,
            version: padding_flags_version & 0b0000_0011,
            token: Token(token),
            response_token: Token(response_token),
        }
    }
    pub fn unpack(self) -> PacketHeaderConnless {
        self.unpack_warn(&mut Ignore)
    }
}

impl PacketHeaderConnless {
    pub fn pack(self) -> PacketHeaderConnlessPacked {
        let PacketHeaderConnless { flags, version, token, response_token } = self;
        // Check that the fields do not exceed their maximal size.
        assert!(flags >> PACKET_FLAGS_BITS == 0);
        assert!(version >> VERSION_BITS == 0);
        PacketHeaderConnlessPacked {
            padding_flags_version: flags << 2 | version as u8,
            token: token.0,
            response_token: response_token.0,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChunkHeader {
    pub flags: u8, // u2
    pub size: u16, // u10
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChunkHeaderVital {
    pub h: ChunkHeader,
    pub sequence: u16, // u16
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ChunkHeaderPacked {
    flags_size: u8, // u2 u6
    padding_size: u8, // u2 u6
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ChunkHeaderVitalPacked {
    flags_size: u8, // u2 u6
    sequence_size: u8, // u2 u6
    sequence: u8,
}

/// -> Some((chunk_header, sequence, rest))
pub fn read_chunk_header<'a, W>(warn: &mut W, data: &'a [u8])
    -> Option<(ChunkHeader, Option<u16>, &'a [u8])>
    where W: Warn<Warning>,
{
    let (raw_header, chunk_data_and_rest) =
        unwrap_or_return!(ChunkHeaderPacked::from_byte_slice(data));

    let header = raw_header.unpack_warn(&mut Ignore);
    Some(if header.flags & CHUNKFLAG_VITAL != 0 {
        let (header, chunk_data_and_rest_vital) =
            unwrap_or_return!(ChunkHeaderVitalPacked::from_byte_slice(data));
        let header = header.unpack_warn(warn);
        (header.h, Some(header.sequence), chunk_data_and_rest_vital)
    } else {
        raw_header.unpack_warn(warn);
        (header, None, chunk_data_and_rest)
    })
}

impl ChunkHeaderPacked {
    pub fn unpack_warn<W: Warn<Warning>>(self, warn: &mut W) -> ChunkHeader {
        let ChunkHeaderPacked { flags_size, padding_size } = self;
        if padding_size & 0b1111_0000 != 0 {
            warn.warn(Warning::ChunkHeaderPadding);
        }
        ChunkHeader {
            flags: (flags_size & 0b1100_0000) >> 6,
            size: ((((flags_size & 0b0011_1111) as u16) << 6)
                | (padding_size & 0b0011_1111) as u16),
        }
    }
    pub fn unpack(self) -> ChunkHeader {
        self.unpack_warn(&mut Ignore)
    }
}

impl ChunkHeader {
    pub fn pack(self) -> ChunkHeaderPacked {
        let ChunkHeader { flags, size } = self;
        // Check that the fields do not exceed their maximal size.
        assert!(flags >> CHUNK_FLAGS_BITS == 0);
        assert!(size >> CHUNK_SIZE_BITS == 0);
        ChunkHeaderPacked {
            flags_size: (flags & 0b11) << 6 | ((size & 0b1111_1100_0000) >> 6) as u8,
            padding_size: (size & 0b0000_0011_1111) as u8
        }
    }
}

impl ChunkHeaderVitalPacked {
    pub fn unpack_warn<W: Warn<Warning>>(self, warn: &mut W) -> ChunkHeaderVital {
        let ChunkHeaderVitalPacked { flags_size, sequence_size, sequence } = self;
        ChunkHeaderVital {
            h: ChunkHeaderPacked {
                flags_size: flags_size,
                padding_size: sequence_size & 0b0011_1111,
            }.unpack_warn(warn),
            sequence: ((sequence_size & 0b1100_0000) as u16) << 2
                | ((sequence & 0b1111_1111) as u16),
        }
    }
    pub fn unpack(self) -> ChunkHeaderVital {
        self.unpack_warn(&mut Ignore)
    }
}

impl ChunkHeaderVital {
    pub fn pack(self) -> ChunkHeaderVitalPacked {
        let ChunkHeaderVital { h, sequence } = self;
        assert!(sequence >> SEQUENCE_BITS == 0);
        let ChunkHeaderPacked { flags_size, padding_size } = h.pack();
        ChunkHeaderVitalPacked {
            flags_size: flags_size,
            sequence_size: (padding_size & 0b0011_1111)
                | ((sequence & 0b11_0000_0000) >> 2) as u8,
            sequence: (sequence & 0b00_1111_1111) as u8,
        }
    }
}

unsafe_boilerplate_packed!(PacketHeaderPacked, HEADER_SIZE, test_ph_size, test_ph_align);
unsafe_boilerplate_packed!(PacketHeaderConnlessPacked, HEADER_SIZE_CONNLESS, test_phc_size, test_phc_align);
unsafe_boilerplate_packed!(ChunkHeaderPacked, CHUNK_HEADER_SIZE, test_ch_size, test_ch_align);
unsafe_boilerplate_packed!(ChunkHeaderVitalPacked, CHUNK_HEADER_SIZE_VITAL, test_chv_size, test_chv_align);

#[cfg(test)]
mod test {
    use super::CHUNK_FLAGS_BITS;
    use super::CHUNK_SIZE_BITS;
    use super::ChunkHeader;
    use super::ChunkHeaderPacked;
    use super::ChunkHeaderVital;
    use super::ChunkHeaderVitalPacked;
    use super::ChunksIter;
    use super::ConnectedPacket;
    use super::ConnectedPacketType;
    use super::MAX_PACKETSIZE;
    use super::PACKET_FLAGS_BITS;
    use super::Packet;
    use super::PacketHeader;
    use super::PacketHeaderPacked;
    use super::PacketReadError::*;
    use super::PacketReadError;
    use super::SEQUENCE_BITS;
    use super::Token;
    use super::Warning::*;
    use super::Warning;
    use warn::Panic;
    use warn::Warn;

    struct WarnVec<'a>(&'a mut Vec<Warning>);

    impl<'a> Warn<Warning> for WarnVec<'a> {
        fn warn(&mut self, warning: Warning) {
            self.0.push(warning);
        }
    }

    fn assert_warnings(input: &[u8], warnings: &[Warning]) {
        let mut vec = vec![];
        let mut buffer = Vec::with_capacity(4096);
        let packet = Packet::read(&mut WarnVec(&mut vec), input, &mut buffer).unwrap();
        if let Packet::Connected(ConnectedPacket {
            type_: ConnectedPacketType::Chunks(_, num_chunks, chunk_data),
            ..
        }) = packet {
            let mut chunks = ChunksIter::new(chunk_data, num_chunks);
            while let Some(_) = chunks.next_warn(&mut WarnVec(&mut vec)) { }
        }
        if warnings != &[ChunksNoChunks] {
            vec.retain(|w| *w != ChunksNoChunks);
        }
        assert_eq!(vec, warnings);
    }

    fn assert_warn(input: &[u8], warning: Warning) {
        assert_warnings(input, &[warning]);
    }

    fn assert_no_warn(input: &[u8]) {
        assert_warnings(input, &[]);
    }

    fn assert_err(input: &[u8], error: PacketReadError) {
        let mut buffer = Vec::with_capacity(4096);
        assert_eq!(Packet::read(&mut Panic, input, &mut buffer).unwrap_err(), error);
    }

    #[test] fn w_chp() { assert_warn(b"\x00\x00\x01\x00\x00\x00\x00\x00\xc0", ChunkHeaderPadding) }
    #[test] fn w_cud1() { assert_warn(b"\x00\x00\x00\x00\x00\x00\x00\xff", ChunksUnknownData) }
    #[test] fn w_cud2() { assert_warn(b"\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00", ChunksUnknownData) }
    #[test] fn w_cud3() { assert_no_warn(b"\x00\x00\x01\x00\x00\x00\x00\x00\x00") }
    #[test] fn w_cud4_ddnet_6() { assert_warn(b"\x00\x00\x01\x00\x00\x12\x34\x45\x67", ChunksUnknownData) }
    #[test] fn w_cnc1() { assert_warn(b"\x00\x00\x01\x00\x00\x00\x00", ChunksNumChunks) }
    #[test] fn w_cnc2() { assert_warn(b"\x00\x00\x00\x00\x00\x00\x00\x00\x00", ChunksNumChunks) }
    #[test] fn w_cnc_() { assert_warn(b"\x00\x00\x00\x00\x00\x00\x00", ChunksNoChunks) }
    #[test] fn w_cf_1() { assert_warn(b"\x31\x00\x00\x00\x00\x00\x00\x00\x00", ConnlessFlags) }
    #[test] fn w_cf_2() { assert_warn(b"\x29\x00\x00\x00\x00\x00\x00\x00\x00", ConnlessFlags) }
    #[test] fn w_cf_3() { assert_warn(b"\x25\x00\x00\x00\x00\x00\x00\x00\x00", ConnlessFlags) }
    #[test] fn w_cf_4() { assert_no_warn(b"\x21\x00\x00\x00\x00\x00\x00\x00\x00") }
    #[test] fn w_ced1() { assert_warn(b"\x04\x00\x00\x00\x00\x00\x00\x00\x00", ControlExcessData) }
    #[test] fn w_ced2() { assert_warn(b"\x04\x00\x00\x00\x00\x00\x00\x04\x00\x00", ControlExcessData) }
    #[test] fn w_cf1() { assert_warn(b"\x14\x00\x00\x00\x00\x00\x00\x15\x37", ControlFlags) }
    #[test] fn w_cf2() { assert_warn(b"\x0c\x00\x00\x00\x00\x00\x00\x00", ControlFlags) }
    #[test] fn w_cnt1() { assert_warn(b"\x04\x00\x00\x00\x00\x00\x00\x04\x01", ControlNulTermination) }
    #[test] fn w_cnt2() { assert_no_warn(b"\x04\x00\x00\x00\x00\x00\x00\x04") }
    #[test] fn w_cnc() { assert_warn(b"\x04\x00\xff\x00\x00\x00\x00\x00", ControlNumChunks) }
    #[test] fn w_php1() { assert_warn(b"\x80\x00\x00\x00\x00\x00\x00", PacketHeaderPadding) }
    #[test] fn w_php2() { assert_warn(b"\x40\x00\x00\x00\x00\x00\x00", PacketHeaderPadding) }

    #[test] fn e_cm() { assert_err(b"\x04\x00\x00\x00\x00\x00\x00", ControlMissing) }
    #[test] fn e_crtm() { assert_err(b"\x04\x00\x00\x00\x00\x00\x00\x05", ControlResponseTokenMissing) }
    #[test] fn e_ctrts() { assert_err(b"\x04\x00\x00\xff\xff\xff\xff\x05\x00\x00\x00\x00", ControlTokenRequestTooShort) }
    #[test] fn e_sc() { assert_err(b"\xff\xff\xff", TooShort) }
    #[test] fn e_tl() { assert_err(&[0; MAX_PACKETSIZE+1], TooLong) }
    #[test] fn e_ts1() { assert_err(b"\x00\x00", TooShort) }
    #[test] fn e_ts2() { assert_err(b"", TooShort) }
    #[test] fn e_uc1() { assert_err(b"\x04\x00\x00\x00\x00\x00\x00\x06", UnknownControl) }
    #[test] fn e_uc2() { assert_err(b"\x04\x00\x00\x00\x00\x00\x00\xff", UnknownControl) }
    #[test] fn e_c() { assert_err(b"\x10\x00\x00\x00\x00\x00\x00", Compression) }
    #[test] fn e_ucv() { assert_err(b"\x22\x00\x00\x00\x00\x00\x00\x00\x00", UnknownConnlessVersion) }

    quickcheck! {
        fn packet_header_roundtrip(flags: u8, ack: u16, num_chunks: u8, token: (u8, u8, u8, u8)) -> bool {
            let flags = flags ^ (flags >> PACKET_FLAGS_BITS << PACKET_FLAGS_BITS);
            let ack = ack ^ (ack >> SEQUENCE_BITS << SEQUENCE_BITS);
            let token = Token([token.0, token.1, token.2, token.3]);
            let packet_header = PacketHeader {
                flags,
                ack,
                num_chunks,
                token,
            };
            packet_header == packet_header.pack().unpack()
        }

        fn packet_header_unpack(v: (u8, u8, u8, u8, u8, u8, u8)) -> bool {
            // Two bits must be zeroed (see doc/packet7.md).
            let v0 = v.0 & 0b0011_1111;
            let bytes = &[v0, v.1, v.2, v.3, v.4, v.5, v.6];
            PacketHeaderPacked::from_bytes(bytes).unpack().pack().as_bytes() == bytes
        }

        fn chunk_header_roundtrip(flags: u8, size: u16, sequence: u16) -> bool {
            let flags = flags ^ (flags >> CHUNK_FLAGS_BITS << CHUNK_FLAGS_BITS);
            let size = size ^ (size >> CHUNK_SIZE_BITS << CHUNK_SIZE_BITS);
            let sequence = sequence ^ (sequence >> SEQUENCE_BITS << SEQUENCE_BITS);
            let chunk_header = ChunkHeader {
                flags: flags,
                size: size,
            };
            let chunk_header_vital = ChunkHeaderVital {
                h: chunk_header,
                sequence: sequence,
            };
            chunk_header == chunk_header.pack().unpack()
                && chunk_header_vital == chunk_header_vital.pack().unpack()
        }

        fn chunk_header_unpack(v: (u8, u8, u8)) -> bool {
            let bytes2 = &[v.0, v.1];
            let bytes3 = &[v.0, v.1, v.2];
            let bytes2_result = &[v.0, v.1 & 0b0011_1111];
            let bytes3_result = &[v.0, v.1, v.2];
            ChunkHeaderPacked::from_bytes(bytes2).unpack().pack().as_bytes() == bytes2_result
                && ChunkHeaderVitalPacked::from_bytes(bytes3).unpack().pack().as_bytes() == bytes3_result
        }
    }
}
