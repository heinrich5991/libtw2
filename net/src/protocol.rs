use arrayvec::ArrayVec;

pub const MAX_PACKETSIZE: usize = 1400;
pub const MAX_PAYLOAD: usize = 1394;

pub type DataBuffer = ArrayVec<[u8; 2048]>;

pub enum ControlPacket {
    KeepAlive,
    Connect,
    ConnectAccept,
    Accept,
    Close(DataBuffer),
}

pub enum Packet {
    Connless(DataBuffer),
    Control(ControlPacket),
    Chunks(DataBuffer),
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PacketHeaderPacked {
    flags_ack: u8, // u4 u4
    ack: u8,
    num_chunks: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    pub flags: u8, // u4
    pub ack: u16, // u12
    pub num_chunks: u8,
}

impl PacketHeaderPacked {
    pub fn unpack(self) -> PacketHeader {
        let PacketHeaderPacked { flags_ack, ack, num_chunks } = self;
        PacketHeader {
            flags: (flags_ack & 0b1111_0000) >> 4,
            ack: (((flags_ack & 0b0000_1111) as u16) << 8) | (ack as u16),
            num_chunks: num_chunks,
        }
    }
}

impl PacketHeader {
    pub fn pack(self) -> PacketHeaderPacked {
        let PacketHeader { flags, ack, num_chunks } = self;
        // Check that the fields do not exceed their maximal size.
        assert!(flags >> 4 == 0);
        assert!(ack >> 12 == 0);
        PacketHeaderPacked {
            flags_ack: flags << 4 | (ack >> 8) as u8,
            ack: ack as u8,
            num_chunks: num_chunks,
        }
    }
}

unsafe_boilerplate_packed!(PacketHeaderPacked, 3, test_ph_size, test_ph_align);

#[cfg(test)]
mod test {
    use super::PacketHeader;
    use super::PacketHeaderPacked;

    #[quickcheck]
    fn packet_header_roundtrip(flags: u8, ack: u16, num_chunks: u8) -> bool {
        let flags = flags ^ (flags >> 4 << 4);
        let ack = ack ^ (ack >> 4 << 4);
        let packet_header = PacketHeader {
            flags: flags,
            ack: ack,
            num_chunks: num_chunks,
        };
        packet_header == packet_header.pack().unpack()
    }

    #[quickcheck]
    fn packet_header_unpack((v0, v1, v2): (u8, u8, u8)) -> bool {
        let bytes = &[v0, v1, v2];
        PacketHeaderPacked::from_bytes(bytes).unpack().pack().as_bytes() == bytes
    }
}
