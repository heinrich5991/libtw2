use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use buffer;
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
pub const HEADER_SIZE: usize = 3;
pub const MAX_PACKETSIZE: usize = 1400;
pub const PADDING_SIZE_CONNLESS: usize = 3;

// For connectionless packets, this is obvious (MAX_PACKETSIZE - HEADER_SIZE -
// PADDING_SIZE_CONNLESS). For packets sent in a connection context, you also
// get a chunk header which replaces the connless padding (it's also 3 bytes
// long).
pub const MAX_PAYLOAD: usize = 1394;

pub const PACKETFLAG_CONTROL:        u8 = 1 << 0;
pub const PACKETFLAG_CONNLESS:       u8 = 1 << 1;
pub const PACKETFLAG_REQUEST_RESEND: u8 = 1 << 2;
pub const PACKETFLAG_COMPRESSION:    u8 = 1 << 3;

pub const CHUNKFLAG_RESEND: u8 = 1 << 1;
pub const CHUNKFLAG_VITAL:  u8 = 1 << 0;

pub const CTRLMSG_KEEPALIVE:     u8 = 0;
pub const CTRLMSG_CONNECT:       u8 = 1;
pub const CTRLMSG_CONNECTACCEPT: u8 = 2;
pub const CTRLMSG_ACCEPT:        u8 = 3;
pub const CTRLMSG_CLOSE:         u8 = 4;

pub const CTRLMSG_CLOSE_REASON_LENGTH: usize = 127;
pub const CHUNK_FLAGS_BITS: u32 = 2;
pub const CHUNK_SIZE_BITS: u32 = 10;
pub const PACKET_FLAGS_BITS: u32 = 4;
pub const SEQUENCE_BITS: u32 = 10;
pub const SEQUENCE_MODULUS: u16 = 1 << SEQUENCE_BITS;

pub fn chunk_header_size(vital: bool) -> usize {
    if vital {
        CHUNK_HEADER_SIZE_VITAL
    } else {
        CHUNK_HEADER_SIZE
    }
}

#[derive(Debug)]
pub enum Error {
    Capacity(buffer::CapacityError),
    TooLongData,
}

impl From<buffer::CapacityError> for Error {
    fn from(e: buffer::CapacityError) -> Error {
        Error::Capacity(e)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Warning {
    ChunkHeaderPadding,
    ChunkHeaderSequence,
    ChunksNoChunks,
    ChunksNumChunks,
    ChunksUnknownData,
    ConnlessPadding,
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
    ShortConnless,
    TooLong,
    TooShort,
    UnknownControl,
}

#[derive(Clone, Copy)]
pub enum ControlPacket<'a> {
    KeepAlive,
    Connect,
    ConnectAccept,
    Accept,
    Close(&'a [u8]),
}

impl<'a> fmt::Debug for ControlPacket<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlPacket::KeepAlive => f.debug_tuple("KeepAlive").finish(),
            ControlPacket::Connect => f.debug_tuple("Connect").finish(),
            ControlPacket::ConnectAccept => f.debug_tuple("ConnectAccept").finish(),
            ControlPacket::Accept => f.debug_tuple("Accept").finish(),
            ControlPacket::Close(reason) =>
                f.debug_tuple("Close").field(&pretty::AlmostString::new(reason)).finish(),
        }
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
        if self.data.len() == 4 && self.num_remaining_chunks == 0 {
            // DDNet token.
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

// TODO: Make this a member function of `Chunk`
// vital: Some((sequence, resend))
pub fn write_chunk<'a, B: Buffer<'a>>(bytes: &[u8], vital: Option<(u16, bool)>, buffer: B)
    -> Result<&'a [u8], buffer::CapacityError>
{
    with_buffer(buffer, |b| write_chunk_impl(bytes, vital, b))
}

pub fn write_chunk_impl<'d, 's>(bytes: &[u8],
                                vital: Option<(u16, bool)>,
                                mut buffer: BufferRef<'d, 's>)
    -> Result<&'d [u8], buffer::CapacityError>
{
    assert!(bytes.len() >> CHUNK_SIZE_BITS == 0);
    let size = bytes.len().assert_u16();

    let (sequence, resend) = vital.unwrap_or((0, false));
    let resend_flag = if resend { CHUNKFLAG_RESEND } else { 0 };
    let vital_flag = if vital.is_some() { CHUNKFLAG_VITAL } else { 0 };
    let flags = vital_flag | resend_flag;

    let header_nonvital = ChunkHeader {
        flags: flags,
        size: size,
    };

    let header1;
    let header2;
    let header: &[u8] = if vital.is_some() {
        header1 = ChunkHeaderVital {
            h: header_nonvital,
            sequence: sequence,
        }.pack();
        header1.as_bytes()
    } else {
        header2 = header_nonvital.pack();
        header2.as_bytes()
    };
    buffer.write(header)?;
    buffer.write(bytes)?;
    Ok(buffer.initialized())
}

fn write_connless_packet<'a, B: Buffer<'a>>(bytes: &[u8], buffer: B)
    -> Result<&'a [u8], Error>
{
    fn inner<'d, 's>(bytes: &[u8], mut buffer: BufferRef<'d, 's>)
        -> Result<&'d [u8], Error>
    {
        if bytes.len() > MAX_PAYLOAD {
            return Err(Error::TooLongData);
        }
        buffer.write(&[b'\xff'; HEADER_SIZE+PADDING_SIZE_CONNLESS])?;
        buffer.write(bytes)?;
        Ok(buffer.initialized())
    }

    with_buffer(buffer, |b| inner(bytes, b))
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
            if payload.len() < PADDING_SIZE_CONNLESS {
                return Err(ShortConnless);
            }
            let (padding, payload) = payload.split_at(PADDING_SIZE_CONNLESS);
            if !padding.iter().all(|&b| b == 0xff)
                || !bytes[..3].iter().all(|&b| b == 0xff)
            {
                warn.warn(Warning::ConnlessPadding);
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
            if control != CTRLMSG_CLOSE && payload.len() != 0 {
                warn.warn(Warning::ControlExcessData);
            }
            let control = match control {
                CTRLMSG_KEEPALIVE => ControlPacket::KeepAlive,
                CTRLMSG_CONNECT => ControlPacket::Connect,
                CTRLMSG_CONNECTACCEPT => ControlPacket::ConnectAccept,
                CTRLMSG_ACCEPT => ControlPacket::Accept,
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
        };
        buffer.write(fake_header.pack().as_bytes()).unwrap();
        HUFFMAN.decompress(payload, &mut buffer)?;

        Ok(buffer.initialized())
    }

    pub fn write<'b, 'c, B1: Buffer<'b>, B2: Buffer<'c>>(&self,
                                                         compression_buffer: B1,
                                                         buffer: B2)
        -> Result<&'c [u8], Error>
    {
        match *self {
            Packet::Connected(ref p) =>
                with_buffer(compression_buffer, |cb|
                    with_buffer(buffer, |b|
                        p.write_impl(cb, b)
                    )
                ),
            Packet::Connless(ref d) => write_connless_packet(d, buffer),
        }
    }
}

impl<'a> ConnectedPacket<'a> {
    pub fn write<'b, 'c, B1: Buffer<'b>, B2: Buffer<'c>>(&self,
                                                         compression_buffer: B1,
                                                         buffer: B2)
        -> Result<&'c [u8], Error>
    {
        with_buffer(compression_buffer, |cb|
            with_buffer(buffer, |b|
                self.write_impl(cb, b)
            )
        )
    }

    fn write_impl<'d1, 's1, 'd2, 's2>(&self,
                                      mut compression_buffer: BufferRef<'d1, 's1>,
                                      mut buffer: BufferRef<'d2, 's2>)
        -> Result<&'d2 [u8], Error>
    {
        match self.type_ {
            ConnectedPacketType::Chunks(request_resend, num_chunks, payload) => {
                assert!(compression_buffer.remaining() >= MAX_PAYLOAD);
                let mut compression = 0;
                let comp_result = HUFFMAN.compress(payload, &mut compression_buffer);
                if comp_result.map(|s| s.len() < payload.len()).unwrap_or(false) {
                    compression = PACKETFLAG_COMPRESSION;
                }
                let request_resend = if request_resend {
                    PACKETFLAG_REQUEST_RESEND
                } else {
                    0
                };
                buffer.write(PacketHeader {
                    flags: request_resend | compression,
                    ack: self.ack,
                    num_chunks: num_chunks,
                }.pack().as_bytes())?;
                buffer.write(if compression != 0 {
                    compression_buffer.initialized()
                } else {
                    payload
                })?;
                Ok(buffer.initialized())
            }
            ConnectedPacketType::Control(c) => {
                c.write(self.ack, buffer)
            }
        }
    }
}

impl<'a> ControlPacket<'a> {
    fn write<'d, 's>(&self, ack: u16, mut buffer: BufferRef<'d, 's>)
        -> Result<&'d [u8], Error>
    {
        buffer.write(PacketHeader {
            flags: PACKETFLAG_CONTROL,
            ack: ack,
            num_chunks: 0,
        }.pack().as_bytes())?;
        let magic = match *self {
            ControlPacket::KeepAlive => CTRLMSG_KEEPALIVE,
            ControlPacket::Connect => CTRLMSG_CONNECT,
            ControlPacket::ConnectAccept => CTRLMSG_CONNECTACCEPT,
            ControlPacket::Accept => CTRLMSG_ACCEPT,
            ControlPacket::Close(..) => CTRLMSG_CLOSE,
        };
        buffer.write(&[magic])?;
        match *self {
            ControlPacket::Close(m) => {
                assert!(m.iter().all(|&b| b != 0));
                buffer.write(m)?;
                buffer.write(&[0])?;
            },
            _ => {},
        }
        let result = buffer.initialized();
        assert!(result.len() <= MAX_PACKETSIZE);
        Ok(result)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PacketHeaderPacked {
    flags_padding_ack: u8, // u4 u2 u2
    ack: u8,
    num_chunks: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PacketHeader {
    pub flags: u8, // u4
    pub ack: u16, // u10
    pub num_chunks: u8,
}

impl PacketHeaderPacked {
    pub fn unpack_warn<W: Warn<Warning>>(self, warn: &mut W) -> PacketHeader {
        let PacketHeaderPacked { flags_padding_ack, ack, num_chunks } = self;
        // First clause checks whether PACKETFLAG_CONNLESS is set.
        if flags_padding_ack & 0b0010_0000 == 0 && flags_padding_ack & 0b0000_1100 != 0 {
            warn.warn(Warning::PacketHeaderPadding);
        }
        PacketHeader {
            flags: (flags_padding_ack & 0b1111_0000) >> 4,
            ack: (((flags_padding_ack & 0b0000_0011) as u16) << 8) | (ack as u16),
            num_chunks: num_chunks,
        }
    }
    pub fn unpack(self) -> PacketHeader {
        self.unpack_warn(&mut Ignore)
    }
}

impl PacketHeader {
    pub fn pack(self) -> PacketHeaderPacked {
        let PacketHeader { flags, ack, num_chunks } = self;
        // Check that the fields do not exceed their maximal size.
        assert!(flags >> PACKET_FLAGS_BITS == 0);
        assert!(ack >> SEQUENCE_BITS == 0);
        PacketHeaderPacked {
            flags_padding_ack: flags << 4 | (ack >> 8) as u8,
            ack: ack as u8,
            num_chunks: num_chunks,
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
    padding_size: u8, // u4 u4
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ChunkHeaderVitalPacked {
    flags_size: u8, // u2 u6
    sequence_size: u8, // u4 u4
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
            size: ((((flags_size & 0b0011_1111) as u16) << 4)
                | (padding_size & 0b0000_1111) as u16),
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
            flags_size: (flags & 0b11) << 6 | ((size & 0b11_1111_0000) >> 4) as u8,
            padding_size: (size & 0b00_0000_1111) as u8
        }
    }
}

impl ChunkHeaderVitalPacked {
    pub fn unpack_warn<W: Warn<Warning>>(self, warn: &mut W) -> ChunkHeaderVital {
        let ChunkHeaderVitalPacked { flags_size, sequence_size, sequence } = self;
        if (sequence_size & 0b0011_0000) >> 4 != (sequence & 0b1100_0000) >> 6 {
            warn.warn(Warning::ChunkHeaderSequence);
        }
        ChunkHeaderVital {
            h: ChunkHeaderPacked {
                flags_size: flags_size,
                padding_size: sequence_size & 0b0000_1111,
            }.unpack_warn(warn),
            sequence: ((sequence_size & 0b1111_0000) as u16) << 2
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
            sequence_size: (padding_size & 0b0000_1111)
                | ((sequence & 0b11_1100_0000) >> 2) as u8,
            sequence: (sequence & 0b00_1111_1111) as u8,
        }
    }
}

unsafe_boilerplate_packed!(PacketHeaderPacked, HEADER_SIZE, test_ph_size, test_ph_align);
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
    use super::Warning::*;
    use super::Warning;
    use warn::Ignore;
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
        let mut buffer= Vec::with_capacity(4096);
        assert_eq!(Packet::read(&mut Panic, input, &mut buffer).unwrap_err(), error);
    }

    #[test] fn w_chp() { assert_warn(b"\x00\x00\x01\x00\xf0", ChunkHeaderPadding) }
    #[test] fn w_chs1() { assert_warn(b"\x00\x00\x01\x40\x20\x00", ChunkHeaderSequence) }
    #[test] fn w_chs2() { assert_warn(b"\x00\x00\x01\x40\x10\x00", ChunkHeaderSequence) }
    #[test] fn w_chs3() { assert_no_warn(b"\x00\x00\x01\x40\x70\xcf") }
    #[test] fn w_cud1() { assert_warn(b"\x00\x00\x00\xff", ChunksUnknownData) }
    #[test] fn w_cud2() { assert_warn(b"\x00\x00\x01\x00\x00\x00", ChunksUnknownData) }
    #[test] fn w_cud3() { assert_no_warn(b"\x00\x00\x01\x00\x00") }
    #[test] fn w_cud4_ddnet() { assert_no_warn(b"\x00\x00\x01\x00\x00\x12\x34\x45\x67") }
    #[test] fn w_cnc1() { assert_warn(b"\x00\x00\x01", ChunksNumChunks) }
    #[test] fn w_cnc2() { assert_warn(b"\x00\x00\x00\x00\x00", ChunksNumChunks) }
    #[test] fn w_cnc_() { assert_warn(b"\x00\x00\x00", ChunksNoChunks) }
    #[test] fn w_cp1() { assert_warn(b"xe\x01\x02\x03\x04", ConnlessPadding) }
    #[test] fn w_cp2() { assert_warn(b"\xff\xff\xff\xff\xff\xfe", ConnlessPadding) }
    #[test] fn w_cp3() { assert_warn(b"\x7f\xff\xff\xff\xff\xff", ConnlessPadding) }
    #[test] fn w_cp4() { assert_no_warn(b"\xff\xff\xff\xff\xff\xff") }
    #[test] fn w_ced1() { assert_warn(b"\x10\x00\x00\x00\x00", ControlExcessData) }
    #[test] fn w_ced2() { assert_warn(b"\x10\x00\x00\x04\x00\x00", ControlExcessData) }
    #[test] fn w_cf1() { assert_warn(b"\x90\x00\x00\x15\x37", ControlFlags) }
    #[test] fn w_cf2() { assert_warn(b"\x50\x00\x00\x00", ControlFlags) }
    #[test] fn w_cnt1() { assert_warn(b"\x10\x00\x00\x04\x01", ControlNulTermination) }
    #[test] fn w_cnt2() { assert_no_warn(b"\x10\x00\x00\x04") }
    #[test] fn w_cnc() { assert_warn(b"\x10\x00\xff\x00", ControlNumChunks) }
    #[test] fn w_php1() { assert_warn(b"\x08\x00\x00", PacketHeaderPadding) }
    #[test] fn w_php2() { assert_warn(b"\x04\x00\x00", PacketHeaderPadding) }

    #[test] fn e_cm() { assert_err(b"\x10\x00\x00", ControlMissing) }
    #[test] fn e_sc() { assert_err(b"\xff\xff\xff", ShortConnless) }
    #[test] fn e_tl() { assert_err(&[0; MAX_PACKETSIZE+1], TooLong) }
    #[test] fn e_ts1() { assert_err(b"\x00\x00", TooShort) }
    #[test] fn e_ts2() { assert_err(b"", TooShort) }
    #[test] fn e_uc1() { assert_err(b"\x10\x00\x00\x05", UnknownControl) }
    #[test] fn e_uc2() { assert_err(b"\x10\x00\x00\xff", UnknownControl) }
    #[test] fn e_c() { assert_err(b"\x80\x00\x00", Compression) }

    quickcheck! {
        fn packet_header_roundtrip(flags: u8, ack: u16, num_chunks: u8) -> bool {
            let flags = flags ^ (flags >> PACKET_FLAGS_BITS << PACKET_FLAGS_BITS);
            let ack = ack ^ (ack >> SEQUENCE_BITS << SEQUENCE_BITS);
            let packet_header = PacketHeader {
                flags: flags,
                ack: ack,
                num_chunks: num_chunks,
            };
            packet_header == packet_header.pack().unpack()
        }

        fn packet_header_unpack(v: (u8, u8, u8)) -> bool {
            // Two bits must be zeroed (see doc/packet.md).
            let v0 = v.0 & 0b1111_0011;
            let bytes = &[v0, v.1, v.2];
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
            let bytes2_result = &[v.0, v.1 & 0b0000_1111];
            let bytes3_result = &[v.0, v.1 | ((v.2 & 0b1100_0000) >> 2), v.2 | ((v.1 & 0b0011_0000) << 2)];
            ChunkHeaderPacked::from_bytes(bytes2).unpack().pack().as_bytes() == bytes2_result
                && ChunkHeaderVitalPacked::from_bytes(bytes3).unpack().pack().as_bytes() == bytes3_result
        }

        fn packet_read_no_panic(data: Vec<u8>) -> bool {
            let mut buffer = [0; MAX_PACKETSIZE];
            let _ = Packet::read(&mut Ignore, &data, &mut buffer[..]);
            true
        }
    }
}
