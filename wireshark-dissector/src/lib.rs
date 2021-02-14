extern crate anyhow;
extern crate arrayvec;
#[macro_use]
extern crate common;
extern crate gamenet_common;
extern crate gamenet_spec;
extern crate huffman;
#[macro_use]
extern crate matches;
extern crate net;
extern crate packer;
extern crate serde_json;
extern crate uuid;
extern crate warn;
extern crate wireshark_dissector_sys as sys;

mod format;
mod intern;
mod spec;

use arrayvec::ArrayVec;
use common::num::Cast;
use common::pretty;
use format::Bitfield;
use format::CommaSeparated;
use format::NumBytes;
use huffman::instances::TEEWORLDS as HUFFMAN;
use intern::Interned;
use intern::intern;
use net::protocol;
use packer::Unpacker;
use gamenet_spec::Identifier;
use spec::Spec;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::os::raw::c_void;
use std::ptr;
use uuid::Uuid;
use warn::Ignore;

const SERIALIZED_SPEC: &'static str = include_str!("../../gamenet/generate/spec/ddnet-15.2.5.json");

const TW_PORT: u32 = 8303;
static mut PROTO_TW_PACKET: c_int = -1;
static mut PROTO_TW_CHUNK: c_int = -1;

static mut ETT_PACKET: c_int = -1;
static mut ETT_PACKET_FLAGS: c_int = -1;
static mut ETT_CHUNK: c_int = -1;
static mut ETT_CHUNK_HEADER: c_int = -1;
static mut ETT_CHUNK_HEADER_FLAGS: c_int = -1;
static mut ETT_MSG_ID: c_int = -1;

static mut HF_PACKET_FLAGS: c_int = -1;
static mut HF_PACKET_CONTROL: c_int = -1;
static mut HF_PACKET_CONNLESS: c_int = -1;
static mut HF_PACKET_REQUEST_RESEND: c_int = -1;
static mut HF_PACKET_COMPRESSION: c_int = -1;
static mut HF_PACKET_ACK: c_int = -1;
static mut HF_PACKET_NUM_CHUNKS: c_int = -1;
static mut HF_PACKET_CTRL: c_int = -1;
static mut HF_PACKET_CTRL_CLOSE_REASON: c_int = -1;
static mut HF_PACKET_PAYLOAD: c_int = -1;
static mut HF_CHUNK_HEADER: c_int = -1;
static mut HF_CHUNK_HEADER_FLAGS: c_int = -1;
static mut HF_CHUNK_HEADER_RESEND: c_int = -1;
static mut HF_CHUNK_HEADER_VITAL: c_int = -1;
static mut HF_CHUNK_HEADER_SIZE: c_int = -1;
static mut HF_CHUNK_HEADER_SEQ: c_int = -1;

static mut SPEC: Option<Spec> = None;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static plugin_want_major: c_int = 3;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static plugin_want_minor: c_int = 4;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static plugin_version: [u8; 6] = *b"0.0.1\0";

#[inline]
fn c(s: &'static str) -> *const c_char {
    intern::intern_static_with_nul(s).c()
}

#[inline]
const fn cc(s: &'static str) -> *const c_char {
    s.as_bytes().as_ptr() as *const _
}

pub const HFRI_DEFAULT: sys::_header_field_info = sys::_header_field_info {
    name: 0 as _,
    abbrev: 0 as _,
    type_: 0,
    display: 0,
    strings: 0 as _,
    bitmask: 0,
    blurb: 0 as _,
    id: -1,
    parent: 0,
    ref_type: 0,
    same_name_prev_id: -1,
    same_name_next: 0 as _,
};

fn unpack_header(data: &[u8]) -> Option<protocol::PacketHeader> {
    let (raw_header, _) =
        protocol::PacketHeaderPacked::from_byte_slice(data)?;
    Some(raw_header.unpack_warn(&mut Ignore))
}

unsafe extern "C" fn dissect_tw(
    tvb: *mut sys::tvbuff_t,
    pinfo: *mut sys::packet_info,
    ttree: *mut sys::proto_tree,
    _data: *mut c_void,
) -> c_int {
    let spec = SPEC.as_ref().unwrap();

    sys::col_set_str((*pinfo).cinfo, sys::COL_PROTOCOL as c_int, c("TW\0"));
    sys::col_clear((*pinfo).cinfo, sys::COL_INFO as c_int);

    let original_tvb = tvb;
    let mut tvb = tvb;
    let len = sys::tvb_reported_length(tvb).usize();
    let mut original_buffer = Vec::with_capacity(len);
    let mut decompress_buffer: ArrayVec<[u8; 2048]> = ArrayVec::new();
    original_buffer.set_len(len);
    sys::tvb_memcpy(tvb, original_buffer.as_mut_ptr() as *mut c_void, 0, len.u64());
    let mut data: &[u8] = &original_buffer;

    // Must be below `let mut tvb = tvb;`
    macro_rules! field {
        ($type:expr, $tree:expr, $hf:expr, $from:expr, $to:expr, $value:expr, $fmt:expr, $($args:tt)*) => {{
            let mut formatted: ArrayVec<[u8; 256]> = ArrayVec::new();
            write!(formatted, $fmt, $($args)*).unwrap();
            formatted.push(0);
            $type($tree, $hf, tvb, $from, $to, $value, cc("%s\0"), CStr::from_bytes_with_nul(&formatted).unwrap().as_ptr())
        }};
    }
    macro_rules! field_none {
        ($tree:expr, $hf:expr, $from:expr, $to:expr, $fmt:expr, $($args:tt)*) => {{
            let mut formatted: ArrayVec<[u8; 256]> = ArrayVec::new();
            write!(formatted, $fmt, $($args)*).unwrap();
            formatted.push(0);
            sys::proto_tree_add_none_format($tree, $hf, tvb, $from, $to, cc("%s\0"), CStr::from_bytes_with_nul(&formatted).unwrap().as_ptr())
        }}
    }
    macro_rules! field_boolean {
        ($tree:expr, $hf:expr, $from:expr, $value:expr, $fmt:expr, $($args:tt)*) => {{
            let value: bool = $value;
            field!(sys::proto_tree_add_boolean_format, $tree, $hf, $from, 1, value as c_uint, $fmt, $($args)*)
        }};
    }
    macro_rules! field_uint {
        ($tree:expr, $hf:expr, $from:expr, $to:expr, $value:expr, $fmt:expr, $($args:tt)*) => {
            field!(sys::proto_tree_add_uint_format, $tree, $hf, $from, $to, $value as c_uint, $fmt, $($args)*)
        };
    }
    macro_rules! field_bytes {
        ($tree:expr, $hf:expr, $from:expr, $to:expr, $fmt:expr, $($args:tt)*) => {{
            field!(sys::proto_tree_add_bytes_format, $tree, $hf, $from, $to, ptr::null(), $fmt, $($args)*)
        }}
    }
    macro_rules! field_string {
        ($tree:expr, $hf:expr, $from:expr, $to:expr, $value:expr, $fmt:expr, $($args:tt)*) => {
            field!(sys::proto_tree_add_string_format, $tree, $hf, $from, $to, $value, $fmt, $($args)*)
        };
    }

    let header = if let Some(h) = unpack_header(data) {
        h
    } else {
        return sys::tvb_reported_length(original_tvb) as c_int;
    };

    let compression = header.flags & protocol::PACKETFLAG_COMPRESSION != 0;
    let request_resend = header.flags & protocol::PACKETFLAG_REQUEST_RESEND != 0;
    let connless = header.flags & protocol::PACKETFLAG_CONNLESS != 0;
    let ctrl = header.flags & protocol::PACKETFLAG_CONTROL != 0;

    let compression = !connless && compression;
    let request_resend = !connless && request_resend;
    let ctrl = !connless && ctrl;

    let header_size = if !connless { 3 } else { 6 };
    let ti = sys::proto_tree_add_item(ttree, PROTO_TW_PACKET, tvb, 0, header_size, sys::ENC_NA);
    let tree = sys::proto_item_add_subtree(ti, ETT_PACKET);

    let mut flags_description: CommaSeparated<[u8; 256]> = CommaSeparated::new();
    if connless {
        flags_description.add("connectionless");
    } else {
        if compression { flags_description.add("compressed"); }
        if request_resend { flags_description.add("resend requested"); }
        if ctrl { flags_description.add("control"); }
    }
    let flags_field = field_uint!(tree, HF_PACKET_FLAGS, 0, 1, header.flags,
        "Flags: {} ({})",
        flags_description.or("none"),
        Bitfield::new(&data[0..1], 0b1111_0000),
    );
    let flag_tree = sys::proto_item_add_subtree(flags_field, ETT_PACKET_FLAGS);

    if !connless {
        field_boolean!(flag_tree, HF_PACKET_COMPRESSION, 0, compression,
            "{} = {}",
            Bitfield::new(&data[0..1], protocol::PACKETFLAG_COMPRESSION.u64() << 4),
            if compression { "Compressed" } else { "Not compressed" },
        );
        field_boolean!(flag_tree, HF_PACKET_REQUEST_RESEND, 0, request_resend,
            "{} = {}",
            Bitfield::new(&data[0..1], protocol::PACKETFLAG_REQUEST_RESEND.u64() << 4),
            if request_resend { "Resend requested" } else { "No resend requested" },
        );
    } else {
        field_boolean!(flag_tree, HF_PACKET_COMPRESSION, 0, compression,
            "{} = Not compressed (implied by being connectionless)",
            Bitfield::new(&data[0..1], 0),
        );
        field_boolean!(flag_tree, HF_PACKET_REQUEST_RESEND, 0, request_resend,
            "{} = No resend requested (implied by being connectionless)",
            Bitfield::new(&data[0..1], 0),
        );
    }
    field_boolean!(flag_tree, HF_PACKET_CONNLESS, 0, connless,
        "{} = {}",
        Bitfield::new(&data[0..1], protocol::PACKETFLAG_CONNLESS.u64() << 4),
        if connless { "Connectionless" } else { "Connection-oriented" },
    );
    if !connless {
        field_boolean!(flag_tree, HF_PACKET_CONTROL, 0, ctrl,
            "{} = {}",
            Bitfield::new(&data[0..1], protocol::PACKETFLAG_CONTROL.u64() << 4),
            if ctrl { "Control message" } else { "Not a control message" },
        );
    } else {
        field_boolean!(flag_tree, HF_PACKET_CONTROL, 0, ctrl,
            "{} = Not a control message (implied by being connectionless)",
            Bitfield::new(&data[0..1], 0),
        );
    }
    if !connless {
        // TODO: Warn if `padding != 0`.
        field_uint!(tree, HF_PACKET_ACK, 0, 2, header.ack,
            "Acknowleged sequence number: {} ({})",
            header.ack,
            Bitfield::new(&data[0..2], 0b0000_0011_1111_1111),
        );
        if !ctrl {
            field_uint!(tree, HF_PACKET_NUM_CHUNKS, 2, 1, header.num_chunks,
                "Number of chunks: {}",
                header.num_chunks,
            );
        }
    }

    field_bytes!(tree, HF_PACKET_PAYLOAD, header_size, -1,
        "{} ({})",
        if !compression { "Payload" } else { "Compressed payload" },
        NumBytes::new(len - header_size.assert_usize()),
    );

    // Decompress the packet on our own, give a fake packet header so the
    // packet decoding code doesn't get confused.
    let fake_header = protocol::PacketHeader {
        flags: header.flags & !protocol::PACKETFLAG_COMPRESSION,
        ack: header.ack,
        num_chunks: header.num_chunks,
    };
    decompress_buffer.extend(fake_header.pack().as_bytes().iter().cloned());
    if compression {
        if let Err(_) = HUFFMAN.decompress(&data[3..], &mut decompress_buffer) {
            return sys::tvb_reported_length(original_tvb) as c_int;
        }
        let buffer = sys::wmem_alloc((*pinfo).pool, decompress_buffer.len().u64()) as *mut u8;
        sys::memcpy(buffer as *mut c_void, decompress_buffer.as_ptr() as *const c_void, decompress_buffer.len().u64());
        tvb = sys::tvb_new_child_real_data(tvb, buffer, decompress_buffer.len().assert_u32(), decompress_buffer.len().assert_i32());
        sys::add_new_data_source(pinfo, tvb, cc("Decompressed Teeworlds packet\0"));
        data = &decompress_buffer;
    }
    tvb = sys::tvb_new_subset_remaining(tvb, header_size);

    let mut buffer: ArrayVec<[u8; 2048]> = ArrayVec::new();
    let packet = if let Ok(p) = protocol::Packet::read(&mut Ignore, data, &mut buffer) {
        p
    } else {
        return sys::tvb_reported_length(original_tvb) as c_int;
    };

    match packet {
        protocol::Packet::Connected(protocol::ConnectedPacket {
            ack: _,
            type_: protocol::ConnectedPacketType::Control(ctrl)
        }) => {
            use protocol::ControlPacket::*;

            let ctrl_raw = data[3];
            let ctrl_str = match ctrl {
                KeepAlive => "Keep alive",
                Connect => "Connect",
                ConnectAccept => "Accept connection",
                Accept => "Acknowledge connection acceptance",
                Close(_) => "Disconnect",
            };
            field_uint!(tree, HF_PACKET_CTRL, 3, 1, ctrl_raw,
                "Control message: {} ({})",
                ctrl_str,
                ctrl_raw,
            );
            if let Close(reason) = ctrl {
                let reason_cstring = CString::new(reason).unwrap();
                field_string!(tree, HF_PACKET_CTRL_CLOSE_REASON, 4, reason.len().assert_i32(),
                    reason_cstring.as_ptr(),
                    "Reason: {:?}",
                    pretty::AlmostString::new(reason),
                );
            }
        },
        protocol::Packet::Connected(protocol::ConnectedPacket {
            ack: _,
            type_: protocol::ConnectedPacketType::Chunks(_, num_chunks, chunks_data),
        }) => {
            let data = &data[3..];
            let mut iter = protocol::ChunksIter::new(chunks_data, num_chunks);
            let mut summaries = String::new();
            while let (offset, Some(_)) = (iter.pos(), iter.next_warn(&mut Ignore)) {
                let (header, sequence, _) = if let Some(s) =
                    protocol::read_chunk_header(&mut Ignore, &chunks_data[offset..])
                {
                    s
                } else {
                    continue;
                };
                let mut flags_description: CommaSeparated<[u8; 256]> =
                    CommaSeparated::new();
                let resend = header.flags & protocol::CHUNKFLAG_RESEND != 0;
                let vital = header.flags & protocol::CHUNKFLAG_VITAL != 0;
                if resend { flags_description.add("re-sent"); }
                if vital { flags_description.add("vital"); }

                let chunk_header_size = 2 + (sequence.is_some() as usize);
                let chunk_size = chunk_header_size + header.size.usize();
                let ti = sys::proto_tree_add_item(ttree, PROTO_TW_CHUNK, tvb, offset.assert_i32(), chunk_size.assert_i32(), sys::ENC_NA);
                let tree = sys::proto_item_add_subtree(ti, ETT_CHUNK);

                let th = field_none!(tree, HF_CHUNK_HEADER, offset.assert_i32(), if vital { 3 } else { 2 },
                    "Header ({})", flags_description.or("none"));
                let header_tree = sys::proto_item_add_subtree(th, ETT_CHUNK_HEADER);

                let flags_field = field_uint!(header_tree, HF_CHUNK_HEADER_FLAGS, offset.assert_i32(), 1, header.flags,
                    "Flags: {} ({})",
                    flags_description.or("none"),
                    Bitfield::new(&data[offset..offset+1], 0b1100_0000),
                );
                let flag_tree = sys::proto_item_add_subtree(flags_field, ETT_CHUNK_HEADER_FLAGS);
                field_boolean!(flag_tree, HF_CHUNK_HEADER_RESEND, 0, resend,
                    "{} = {}",
                    Bitfield::new(&data[offset..offset+1], protocol::CHUNKFLAG_RESEND.u64() << 6),
                    if ctrl { "Was re-sent" } else { "Was sent for the first time" },
                );
                field_boolean!(flag_tree, HF_CHUNK_HEADER_VITAL, 0, vital,
                    "{} = {}",
                    Bitfield::new(&data[offset..offset+1], protocol::CHUNKFLAG_VITAL.u64() << 6),
                    if vital { "Will be transferred reliably" } else { "Will not be transferred reliably" },
                );
                field_uint!(header_tree, HF_CHUNK_HEADER_SIZE, offset.assert_i32(), 2, header.size,
                    "Size: {} ({})",
                    NumBytes::new(header.size.usize()),
                    Bitfield::new(&data[offset..offset+2], 0b0011_1111_0000_1111),
                );
                if let Some(s) = sequence {
                    field_uint!(header_tree, HF_CHUNK_HEADER_SEQ, offset.assert_i32() + 1, 2, s,
                        "Sequence number: {} ({})",
                        s,
                        Bitfield::new(&data[offset+1..offset+3], 0b1100_0000_1111_1111),
                    );
                }

                let mut p = Unpacker::new(&data[..offset+chunk_size]);
                p.read_raw(offset).unwrap();
                p.read_raw(chunk_header_size).unwrap();
                let mut first_summary = true;
                spec.dissect(tree, tvb, &mut p,
                    &mut |summary| {
                        if !summaries.is_empty() {
                            summaries.push_str(", ");
                        }
                        let summary_c = CString::new(summary).unwrap();
                        sys::proto_item_append_text(ti, c("%s%s\0"),
                            if first_summary { c(": \0") } else { c(", \0") },
                            summary_c.as_ptr(),
                        );
                        first_summary = false;
                        summaries.push_str(summary);
                    }
                );
            }
            let info = CString::new(summaries).unwrap();
            sys::col_add_str((*pinfo).cinfo, sys::COL_INFO as c_int, info.as_ptr());
        }
        protocol::Packet::Connless(_message) => {
            sys::col_set_str((*pinfo).cinfo, sys::COL_INFO as c_int, c("connless\0"));
        },
    }

    sys::tvb_reported_length(original_tvb) as c_int
}

unsafe extern "C" fn proto_register_teeworlds() {
    assert!(SPEC.replace(load_spec().unwrap()).is_none());

    static mut PACKET_HF: [sys::hf_register_info; 10] = unsafe {[
        sys::hf_register_info {
            p_id: &HF_PACKET_FLAGS as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Flags\0"),
                abbrev: cc("tw.packet.flags\0"),
                type_: sys::FT_UINT16,
                display: sys::BASE_HEX as c_int,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_COMPRESSION as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Compressed\0"),
                abbrev: cc("tw.packet.flags.compression\0"),
                type_: sys::FT_BOOLEAN,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_REQUEST_RESEND as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Request resend\0"),
                abbrev: cc("tw.packet.flags.request_resend\0"),
                type_: sys::FT_BOOLEAN,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_CONNLESS as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Connless\0"),
                abbrev: cc("tw.packet.flags.connless\0"),
                type_: sys::FT_BOOLEAN,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_CONTROL as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Control\0"),
                abbrev: cc("tw.packet.flags.control\0"),
                type_: sys::FT_BOOLEAN,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_ACK as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Acknowledged sequence number\0"),
                abbrev: cc("tw.packet.ack\0"),
                type_: sys::FT_UINT16,
                display: sys::BASE_DEC as c_int,
                bitmask: 0,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_NUM_CHUNKS as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Number of chunks\0"),
                abbrev: cc("tw.packet.num_chunks\0"),
                type_: sys::FT_UINT8,
                display: sys::BASE_DEC as c_int,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_CTRL as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Control message\0"),
                abbrev: cc("tw.packet.ctrl\0"),
                type_: sys::FT_UINT8,
                display: sys::BASE_DEC as c_int,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_CTRL_CLOSE_REASON as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Close reason\0"),
                abbrev: cc("tw.packet.ctrl.close_reason\0"),
                type_: sys::FT_STRING,
                display: sys::STR_ASCII as c_int,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_PACKET_PAYLOAD as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: cc("Payload\0"),
                abbrev: cc("tw.packet.payload\0"),
                type_: sys::FT_BYTES,
                ..HFRI_DEFAULT
            },
        },
    ]};

    static mut ETT: [*mut c_int; 6] = unsafe {[
        &ETT_PACKET as *const _ as *mut _,
        &ETT_PACKET_FLAGS as *const _ as *mut _,
        &ETT_CHUNK as *const _ as *mut _,
        &ETT_CHUNK_HEADER as *const _ as *mut _,
        &ETT_CHUNK_HEADER_FLAGS as *const _ as *mut _,
        &ETT_MSG_ID as *const _ as *mut _,
    ]};

    PROTO_TW_PACKET = sys::proto_register_protocol(
        cc("Teeworlds Protocol packet\0"),
        cc("Teeworlds packet\0"),
        cc("twp\0"),
    );
    sys::proto_register_field_array(PROTO_TW_PACKET, PACKET_HF.as_mut_ptr(), PACKET_HF.len().assert_i32());
    register_chunk_protocol(SPEC.as_ref().unwrap());
    sys::proto_register_subtree_array(ETT.as_ptr(), ETT.len().assert_i32());
}

unsafe extern "C" fn proto_reg_handoff_teeworlds() {
    let tw_packet = sys::create_dissector_handle(Some(dissect_tw), PROTO_TW_PACKET);
    sys::dissector_add_uint(cc("udp.port\0"), TW_PORT, tw_packet);
}

trait IdentifierEx {
    fn _identifier(&self) -> &Identifier;
    fn isnake(&self) -> Interned {
        intern(&self._identifier().snake())
    }
    fn idesc(&self) -> Interned {
        intern(&self._identifier().desc())
    }
}
impl IdentifierEx for Identifier {
    fn _identifier(&self) -> &Identifier {
        self
    }
}

fn load_spec() -> anyhow::Result<spec::Spec> {
    spec::Spec::load(SERIALIZED_SPEC)
}

fn to_guid(uuid: Uuid) -> sys::e_guid_t {
    let (data1, data2, data3, &data4) = uuid.as_fields();
    sys::e_guid_t {
        data1,
        data2,
        data3,
        data4,
    }
}

fn register_chunk_protocol(spec: &Spec) {
    unsafe {
        PROTO_TW_CHUNK = sys::proto_register_protocol(
            cc("Teeworlds Protocol chunk\0"),
            cc("Teeworlds chunk\0"),
            cc("tw\0"),
        );
    }
    let mut fields_info = Vec::new();
    let mut etts = Vec::new();
    fields_info.extend_from_slice(&unsafe {[
        sys::hf_register_info {
            p_id: &HF_CHUNK_HEADER as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: c("Header\0"),
                abbrev: c("tw.chunk\0"),
                type_: sys::FT_NONE,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_CHUNK_HEADER_FLAGS as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: c("Flags\0"),
                abbrev: c("tw.chunk.flags\0"),
                type_: sys::FT_UINT8,
                display: sys::BASE_DEC as c_int,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_CHUNK_HEADER_RESEND as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: c("Resend\0"),
                abbrev: c("tw.chunk.flags.resend\0"),
                type_: sys::FT_BOOLEAN,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_CHUNK_HEADER_VITAL as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: c("Vital\0"),
                abbrev: c("tw.chunk.flags.vital\0"),
                type_: sys::FT_BOOLEAN,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_CHUNK_HEADER_SIZE as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: c("Size\0"),
                abbrev: c("tw.chunk.size\0"),
                type_: sys::FT_UINT16,
                display: sys::BASE_DEC as c_int,
                ..HFRI_DEFAULT
            },
        },
        sys::hf_register_info {
            p_id: &HF_CHUNK_HEADER_SEQ as *const _ as *mut _,
            hfinfo: sys::_header_field_info {
                name: c("Sequence number\0"),
                abbrev: c("tw.chunk.seq\0"),
                type_: sys::FT_UINT16,
                display: sys::BASE_DEC as c_int,
                ..HFRI_DEFAULT
            },
        },
    ]});
    spec.field_register_info(
        &mut |hfri| fields_info.push(hfri),
        &mut |ett| etts.push(ett),
    );
    let fields_info = Box::leak(fields_info.into_boxed_slice());
    let etts = Box::leak(etts.into_boxed_slice());
    unsafe {
        sys::proto_register_field_array(PROTO_TW_CHUNK, fields_info.as_mut_ptr(), fields_info.len().assert_i32());
        sys::proto_register_subtree_array(etts.as_ptr(), etts.len().assert_i32());
    }
}

#[no_mangle]
pub unsafe extern "C" fn plugin_register() {
    sys::proto_register_plugin(&sys::proto_plugin {
        register_protoinfo: Some(proto_register_teeworlds),
        register_handoff: Some(proto_reg_handoff_teeworlds),
    });
}

#[cfg(test)]
mod test {
    #[test]
    fn spec_valid() {
        super::load_spec();
    }
}
