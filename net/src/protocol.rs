use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use buffer;
use huffman::instances::TEEWORLDS as HUFFMAN;
use huffman;
use num::ToPrimitive;

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

pub const CHUNK_FLAGS_BITS: u32 = 2;
pub const CHUNK_SIZE_BITS: u32 = 10;
pub const PACKET_FLAGS_BITS: u32 = 4;
pub const SEQUENCE_BITS: u32 = 10;

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

#[derive(Clone, Copy)]
pub enum ControlPacket<'a> {
    KeepAlive,
    Connect,
    ConnectAccept,
    Accept,
    Close(&'a [u8]),
}

#[derive(Clone, Copy)]
pub struct ConnectedPacket<'a> {
    pub ack: u16, // u10
    pub type_: ConnectedPacketType<'a>,
}

#[derive(Clone, Copy)]
pub enum ConnectedPacketType<'a> {
    // Chunks(request_resend, num_chunks, payload)
    Chunks(bool, u8, &'a [u8]),
    Control(ControlPacket<'a>),
}

#[derive(Clone, Copy)]
pub enum Packet<'a> {
    Connless(&'a [u8]),
    Connected(ConnectedPacket<'a>),
}

pub struct Chunk<'a> {
    pub data: &'a [u8],
    // vital: Some((sequence, resend))
    pub vital: Option<(u16, bool)>,
}

#[derive(Clone)]
pub struct ChunksIter<'a> {
    data: &'a [u8],
}

impl<'a> ChunksIter<'a> {
    pub fn new(data: &'a [u8]) -> ChunksIter<'a> {
        ChunksIter {
            data: data,
        }
    }
}

impl<'a> Iterator for ChunksIter<'a> {
    type Item = Chunk<'a>;
    fn next(&mut self) -> Option<Chunk<'a>> {
        if self.data.len() == 0 {
            return None;
        }
        let (header, data) = unwrap_or_return!(ChunkHeaderPacked::from_byte_slice(self.data));
        Some(if header.unpack().flags & CHUNKFLAG_VITAL != 0 {
            let (header, data) = unwrap_or_return!(ChunkHeaderVitalPacked::from_byte_slice(self.data));
            let header = header.unpack();
            self.data = data;
            Chunk {
                data: data,
                vital: Some((header.sequence, header.h.flags & CHUNKFLAG_RESEND != 0)),
            }
        } else {
            self.data = data;
            Chunk {
                data: data,
                vital: None,
            }
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.clone().count();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for ChunksIter<'a> { }

pub fn compress<'a, B: Buffer<'a>>(bytes: &[u8], buffer: B)
    -> Result<&'a [u8], buffer::CapacityError>
{
    HUFFMAN.compress(bytes, buffer)
}

pub fn decompress<'a, B: Buffer<'a>>(bytes: &[u8], buffer: B)
    -> Result<&'a [u8], huffman::DecompressionError>
{
    HUFFMAN.decompress(bytes, buffer)
}

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
    let size = bytes.len().to_u16().unwrap();

    let (sequence, resend) = vital.unwrap_or((0, false));
    let resend_flag = if resend { CHUNKFLAG_RESEND } else { 0 };
    let vital_flag = if vital.is_some() { CHUNKFLAG_VITAL } else { 0 };
    let flags = vital_flag | resend_flag;

    let header_non_vital = ChunkHeader {
        flags: flags,
        size: size,
    };

    let header1;
    let header2;
    let header: &[u8] = if vital.is_some() {
        header1 = ChunkHeaderVital {
            h: header_non_vital,
            sequence: sequence,
        }.pack();
        header1.as_bytes()
    } else {
        header2 = header_non_vital.pack();
        header2.as_bytes()
    };
    try!(buffer.write(header));
    try!(buffer.write(bytes));
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
        try!(buffer.write(&[b'\xff'; HEADER_SIZE+PADDING_SIZE_CONNLESS]));
        try!(buffer.write(bytes));
        Ok(buffer.initialized())
    }

    with_buffer(buffer, |b| inner(bytes, b))
}

impl<'a> Packet<'a> {
    pub fn read<'b, B: Buffer<'b>>(bytes: &'b [u8], buffer: B) -> Option<Packet<'b>> {
        with_buffer(buffer, |b| Packet::read_impl(bytes, b))
    }
    fn read_impl<'d, 's>(bytes: &'d [u8], mut buffer: BufferRef<'d, 's>)
        -> Option<Packet<'d>>
    {
        if bytes.len() > MAX_PACKETSIZE {
            return None;
        }
        let (header, payload) = unwrap_or_return!(PacketHeaderPacked::from_byte_slice(bytes));
        let header = header.unpack();
        // TODO: Maybe warn on "interesting" bytes here.
        if header.flags & PACKETFLAG_CONNLESS != 0 {
            if payload.len() < PADDING_SIZE_CONNLESS {
                return None;
            }
            let payload = &payload[PADDING_SIZE_CONNLESS..];
            return Some(Packet::Connless(payload));
        }

        let payload = if header.flags & PACKETFLAG_COMPRESSION != 0 {
            unwrap_or_return!(decompress(payload, &mut buffer).ok())
        } else {
            payload
        };

        if payload.len() > MAX_PAYLOAD {
            return None;
        }

        let ack = header.ack;
        let type_ = if header.flags & PACKETFLAG_CONTROL != 0 {
            // TODO: Check that header.num_chunks is 0.
            // TODO: Vanilla recognizes PACKETFLAG_COMPRESSION and
            //       PACKETFLAG_REQUEST_RESEND for PACKETFLAG_CONTROL, but does
            //       not set them. What should we do?

            if payload.len() < 1 {
                return None;
            }

            let control = payload[0];
            let payload = &payload[1..];
            let control = match control {
                CTRLMSG_KEEPALIVE => ControlPacket::KeepAlive,
                CTRLMSG_CONNECT => ControlPacket::Connect,
                CTRLMSG_CONNECTACCEPT => ControlPacket::ConnectAccept,
                CTRLMSG_ACCEPT => ControlPacket::Accept,
                // TODO: Check for length
                CTRLMSG_CLOSE => ControlPacket::Close(payload),
                _ => return None, // Unrecognized control packet.
            };

            ConnectedPacketType::Control(control)
        } else {
            let request_resend = header.flags & PACKETFLAG_REQUEST_RESEND != 0;
            ConnectedPacketType::Chunks(request_resend, header.num_chunks, payload)
        };

        Some(Packet::Connected(ConnectedPacket {
            ack: ack,
            type_: type_,
        }))
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
                let comp_result = compress(payload, &mut compression_buffer);
                if comp_result.map(|s| s.len() < payload.len()).unwrap_or(false) {
                    compression = PACKETFLAG_COMPRESSION;
                }
                let request_resend = if request_resend {
                    PACKETFLAG_REQUEST_RESEND
                } else {
                    0
                };
                try!(buffer.write(PacketHeader {
                    flags: request_resend | compression,
                    ack: self.ack,
                    num_chunks: num_chunks,
                }.pack().as_bytes()));
                try!(buffer.write(if compression != 0 {
                    compression_buffer.initialized()
                } else {
                    payload
                }));
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
        try!(buffer.write(PacketHeader {
            flags: PACKETFLAG_CONTROL,
            ack: ack,
            num_chunks: 0,
        }.pack().as_bytes()));
        let magic = match *self {
            ControlPacket::KeepAlive => CTRLMSG_KEEPALIVE,
            ControlPacket::Connect => CTRLMSG_CONNECT,
            ControlPacket::ConnectAccept => CTRLMSG_CONNECTACCEPT,
            ControlPacket::Accept => CTRLMSG_ACCEPT,
            ControlPacket::Close(..) => CTRLMSG_CLOSE,
        };
        try!(buffer.write(&[magic]));
        match *self {
            // TODO: null termination
            ControlPacket::Close(m) => try!(buffer.write(m)),
            _ => {},
        }
        Ok(buffer.initialized())
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
    pub fn unpack(self) -> PacketHeader {
        let PacketHeaderPacked { flags_padding_ack, ack, num_chunks } = self;
        PacketHeader {
            flags: (flags_padding_ack & 0b1111_0000) >> 4,
            ack: (((flags_padding_ack & 0b0000_0011) as u16) << 8) | (ack as u16),
            num_chunks: num_chunks,
        }
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
    flags: u8, // u2
    size: u16, // u10
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChunkHeaderVital {
    h: ChunkHeader,
    sequence: u16, // u16
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

impl ChunkHeaderPacked {
    pub fn unpack(self) -> ChunkHeader {
        let ChunkHeaderPacked { flags_size, padding_size } = self;
        ChunkHeader {
            flags: (flags_size & 0b1100_0000) >> 6,
            size: ((((flags_size & 0b0011_1111) as u16) << 4)
                | (padding_size & 0b0000_1111) as u16),
        }
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
    pub fn unpack(self) -> ChunkHeaderVital {
        let ChunkHeaderVitalPacked { flags_size, sequence_size, sequence } = self;
        ChunkHeaderVital {
            h: ChunkHeaderPacked {
                flags_size: flags_size,
                padding_size: sequence_size & 0b0000_1111,
            }.unpack(),
            sequence: ((sequence_size & 0b1111_0000) as u16) << 2
                | ((sequence & 0b1111_1111) as u16),
        }
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
unsafe_boilerplate_packed!(ChunkHeaderPacked, 2, test_ch_size, test_ch_align);
unsafe_boilerplate_packed!(ChunkHeaderVitalPacked, 3, test_chv_size, test_chv_align);

#[cfg(test)]
mod test {
    use super::CHUNK_FLAGS_BITS;
    use super::CHUNK_SIZE_BITS;
    use super::ChunkHeader;
    use super::ChunkHeaderPacked;
    use super::ChunkHeaderVital;
    use super::ChunkHeaderVitalPacked;
    use super::MAX_PACKETSIZE;
    use super::PACKET_FLAGS_BITS;
    use super::Packet;
    use super::PacketHeader;
    use super::PacketHeaderPacked;
    use super::SEQUENCE_BITS;

    use common::buffer::SliceBuffer;

    #[quickcheck]
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

    #[quickcheck]
    fn packet_header_unpack((v0, v1, v2): (u8, u8, u8)) -> bool {
        // Two bits must be zeroed (see doc/packet.md).
        let v0 = v0 & 0b1111_0011;
        let bytes = &[v0, v1, v2];
        PacketHeaderPacked::from_bytes(bytes).unpack().pack().as_bytes() == bytes
    }

    #[quickcheck]
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

    #[quickcheck]
    fn chunk_header_unpack((v0, v1, v2): (u8, u8, u8)) -> bool {
        let bytes2 = &[v0, v1];
        let bytes3 = &[v0, v1, v2];
        let bytes2_result = &[v0, v1 & 0b0000_1111];
        let bytes3_result = &[v0, v1 | ((v2 & 0b1100_0000) >> 2), v2 | ((v1 & 0b0011_0000) << 2)];
        ChunkHeaderPacked::from_bytes(bytes2).unpack().pack().as_bytes() == bytes2_result
            && ChunkHeaderVitalPacked::from_bytes(bytes3).unpack().pack().as_bytes() == bytes3_result
    }

    #[quickcheck]
    fn packet_read_no_panic(data: Vec<u8>) -> bool {
        let mut buffer = [0; MAX_PACKETSIZE];
        let mut buffer = SliceBuffer::new(&mut buffer);
        Packet::read(&data, &mut buffer);
        true
    }
}
