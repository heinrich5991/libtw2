use common::Buffer;
use common::buffer;
use huffman::instances::TEEWORLDS as HUFFMAN;
use huffman;

pub const MAX_PACKETSIZE: usize = 1400;
pub const HEADER_SIZE: usize = 3;
pub const PADDING_SIZE_CONNLESS: usize = 3;
pub const MAX_PAYLOAD: usize = 1394;

pub const PACKETFLAG_CONTROL:        u8 = 1 << 0;
pub const PACKETFLAG_CONNLESS:       u8 = 1 << 1;
pub const PACKETFLAG_REQUEST_RESEND: u8 = 1 << 2;
pub const PACKETFLAG_COMPRESSION:    u8 = 1 << 3;

pub const CTRLMSG_KEEPALIVE:     u8 = 0;
pub const CTRLMSG_CONNECT:       u8 = 1;
pub const CTRLMSG_CONNECTACCEPT: u8 = 2;
pub const CTRLMSG_ACCEPT:        u8 = 3;
pub const CTRLMSG_CLOSE:         u8 = 4;

pub const CHUNK_FLAGS_BITS: u32 = 2;
pub const CHUNK_SIZE_BITS: u32 = 10;
pub const PACKET_FLAGS_BITS: u32 = 4;
pub const SEQUENCE_BITS: u32 = 10;

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
    // TODO: Control packets don't contain request_resend in vanilla.
    pub request_resend: bool,
    pub ack: u16, // u10
    pub type_: ConnectedPacketType<'a>,
}

#[derive(Clone, Copy)]
pub enum ConnectedPacketType<'a> {
    // Chunks(num_chunks, payload)
    Chunks(u8, &'a [u8]),
    Control(ControlPacket<'a>),
}

#[derive(Clone, Copy)]
pub enum Packet<'a> {
    Connless(&'a [u8]),
    Connected(ConnectedPacket<'a>),
}

pub fn compress(bytes: &[u8], buffer: &mut Buffer)
    -> Result<(), buffer::CapacityError>
{
    HUFFMAN.compress(bytes, buffer)
}

pub fn decompress(bytes: &[u8], buffer: &mut Buffer)
    -> Result<(), huffman::DecompressionError>
{
    HUFFMAN.decompress(bytes, buffer)
}

fn write_connless_packet(bytes: &[u8], buffer: &mut Buffer)
    -> Result<(), Error>
{
    if bytes.len() > MAX_PAYLOAD {
        return Err(Error::TooLongData);
    }
    try!(buffer.write(&[b'\xff'; HEADER_SIZE+PADDING_SIZE_CONNLESS]));
    try!(buffer.write(bytes));
    Ok(())
}

impl<'a> Packet<'a> {
    pub fn read(bytes: &'a [u8], buffer: &'a mut Buffer<'a>) -> Option<Packet<'a>> {
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
            unwrap_or_return!(decompress(payload, buffer).ok());
            buffer.init()
        } else {
            payload
        };

        if payload.len() > MAX_PAYLOAD {
            return None;
        }

        let request_resend = header.flags & PACKETFLAG_REQUEST_RESEND != 0;
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
            ConnectedPacketType::Chunks(header.num_chunks, payload)
        };

        Some(Packet::Connected(ConnectedPacket {
            request_resend: request_resend,
            ack: ack,
            type_: type_,
        }))
    }
    pub fn write(&self, compression_buffer: &mut Buffer, buffer: &mut Buffer)
        -> Result<(), Error>
    {
        match *self {
            Packet::Connected(ref p) => p.write(compression_buffer, buffer),
            Packet::Connless(ref d) => write_connless_packet(d, buffer),
        }
    }
}

impl<'a> ConnectedPacket<'a> {
    fn write(&self, compression_buffer: &mut Buffer, buffer: &mut Buffer)
        -> Result<(), Error>
    {
        match self.type_ {
            ConnectedPacketType::Chunks(num_chunks, payload) => {
                assert!(compression_buffer.len() >= MAX_PAYLOAD);
                let mut compression = 0;
                if let Ok(()) = compress(payload, compression_buffer) {
                    if compression_buffer.len() < payload.len() {
                        compression = PACKETFLAG_COMPRESSION;
                    }
                }
                let request_resend = if self.request_resend {
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
                    &compression_buffer
                } else {
                    payload
                }));
                Ok(())
            }
            ConnectedPacketType::Control(c) => {
                c.write(self.request_resend, self.ack, buffer)
            }
        }
    }
}

impl<'a> ControlPacket<'a> {
    fn write(&self, request_resend: bool, ack: u16, buffer: &mut Buffer)
        -> Result<(), Error>
    {
        let request_resend = if request_resend { PACKETFLAG_REQUEST_RESEND } else { 0 };
        try!(buffer.write(PacketHeader {
            flags: PACKETFLAG_CONTROL | request_resend,
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
            ControlPacket::Close(m) => try!(buffer.write(m)),
            _ => {},
        }
        Ok(())
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PacketHeaderPacked {
    flags_padding_ack: u8, // u4 u2 u2
    ack: u8,
    num_chunks: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
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

#[derive(Copy, Clone)]
pub struct ChunkHeader {
    flags: u8, // u2
    size: u16, // u10
}

#[derive(Copy, Clone)]
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
    use super::Packet;
    use super::PacketHeader;
    use super::PacketHeaderPacked;
    use super::SEQUENCE_BITS;

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
        packet_header == packet_header.pack().unpack()
            && chunk_header_vital == chunk_header_vital.pack().unpack()
    }

    #[quickcheck]
    fn packet_header_unpack((v0, v1, v2): (u8, u8, u8)) -> bool {
        let bytes2 = &[v0, v1];
        let bytes3 = &[v0, v1, v2];
        let bytes2_result = &[v0, v1 & 0b1111_0000];
        ChunkHeaderPacked::from_bytes(bytes2).unpack().pack().as_bytes() == bytes2_result
            && ChunkHeaderVitalPacked::from_bytes(bytes3).unpack().pack().as_bytes() == bytes3
    }

    #[quickcheck]
    fn packet_read_no_panic(data: Vec<u8>) -> bool {
        Packet::read(&data);
        true
    }
}
