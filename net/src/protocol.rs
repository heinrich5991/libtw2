use arrayvec::ArrayVec;

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

pub const FLAGS_BITS: u32 = 4;
pub const SEQUENCE_BITS: u32 = 10;

pub const FLAGS_MASK: u8 = (1 << FLAGS_BITS) - 1;
pub const SEQUENCE_MASK: u8 = (1 << FLAGS_BITS) - 1;

pub type DataBuffer = ArrayVec<[u8; 2048]>;
pub type CloseMsg = ArrayVec<[u8; 128]>;

pub enum ControlPacket {
    KeepAlive,
    Connect,
    ConnectAccept,
    Accept,
    Close(CloseMsg),
}

pub struct ConnectedPacket {
    pub request_resend: bool,
    pub ack: u16, // u10
    pub type_: ConnectedPacketType,
}

pub enum ConnectedPacketType {
    Chunks(DataBuffer),
    Control(ControlPacket),
}

pub enum Packet {
    Connless(DataBuffer),
    Connected(ConnectedPacket),
}

// TODO: Implement compression.
pub fn compress(bytes: &[u8]) -> Result<DataBuffer,()> {
    Err(())
}

pub fn decompress(bytes: &[u8]) -> Result<DataBuffer,()> {
    Err(())
}

fn write_connless_packet(bytes: &[u8]) -> Result<DataBuffer,()> {
    if bytes.len() > MAX_PAYLOAD {
        return Err(());
    }
    let mut result = DataBuffer::new();
    result.extend((0..(HEADER_SIZE+PADDING_SIZE_CONNLESS)).map(|_| b'\xff'));
    result.extend(bytes.iter().cloned());
    Ok(result)
}

impl Packet {
    pub fn read(bytes: &[u8]) -> Option<Packet> {
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
            return Some(Packet::Connless(payload.iter().cloned().collect()));
        }

        let mut payload = if header.flags & PACKETFLAG_COMPRESSION != 0 {
            unwrap_or_return!(decompress(payload).ok())
        } else {
            payload.iter().cloned().collect()
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

            let control = unwrap_or_return!(payload.remove(0));
            let control = match control {
                CTRLMSG_KEEPALIVE => ControlPacket::KeepAlive,
                CTRLMSG_CONNECT => ControlPacket::Connect,
                CTRLMSG_CONNECTACCEPT => ControlPacket::ConnectAccept,
                CTRLMSG_ACCEPT => ControlPacket::Accept,
                // TODO: Check for length
                CTRLMSG_CLOSE => ControlPacket::Close(payload.into_iter().collect()),
                _ => return None, // Unrecognized control packet.
            };

            ConnectedPacketType::Control(control)
        } else {
            ConnectedPacketType::Chunks(payload)
        };

        Some(Packet::Connected(ConnectedPacket {
            request_resend: request_resend,
            ack: ack,
            type_: type_,
        }))
    }
    pub fn write(&self) -> Result<DataBuffer,()> {
        match *self {
            Packet::Connected(ref p) => p.write(),
            Packet::Connless(ref d) => write_connless_packet(d),
        }
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
        assert!(flags >> FLAGS_BITS == 0);
        assert!(ack >> SEQUENCE_BITS == 0);
        PacketHeaderPacked {
            flags_padding_ack: flags << 4 | (ack >> 8) as u8,
            ack: ack as u8,
            num_chunks: num_chunks,
        }
    }
}

unsafe_boilerplate_packed!(PacketHeaderPacked, HEADER_SIZE, test_ph_size, test_ph_align);

#[cfg(test)]
mod test {
    use super::Packet;
    use super::PacketHeader;
    use super::PacketHeaderPacked;
    use super::SEQUENCE_BITS;

    #[quickcheck]
    fn packet_header_roundtrip(flags: u8, ack: u16, num_chunks: u8) -> bool {
        let flags = flags ^ (flags >> 4 << 4);
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
    fn packet_read_no_panic(data: Vec<u8>) -> bool {
        Packet::read(&data);
        true
    }
}
