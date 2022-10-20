use IdentifierEx;
use anyhow::Context as _;
use anyhow::anyhow;
use anyhow::bail;
use arrayvec::ArrayVec;
use common::digest;
use common::num::BeU16;
use common::num::Cast;
use common::pretty::AlmostString;
use crate::HFRI_DEFAULT;
use crate::c;
use crate::to_guid;
use format::Bitfield;
use format::CommaSeparated;
use format::NumBytes;
use gamenet_spec::MessageId;
use intern::Interned;
use intern::intern;
use intern::intern_static_with_nul;
use packer::Unpacker;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::io::Write;
use std::mem;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::rc::Rc;
use std::str;
use warn::Ignore;

#[derive(Debug)]
pub struct Spec {
    pub prefix: Interned,
    pub game_messages: HashMap<MessageId, Message>,
    pub system_messages: HashMap<MessageId, Message>,
    pub connless_messages: HashMap<[u8; 8], Message>,
    pub tree_msg: FieldId,
    pub id_msg: FieldId,
    pub id_msg_system: FieldId,
    pub id_msg_id_raw: FieldId,
    pub id_msg_id_ex: FieldId,
    pub id_connless_id_raw: FieldId,
}
#[derive(Debug)]
pub struct Enumeration {
    pub values: HashMap<i32, Interned>,
}
#[derive(Debug)]
pub struct Flags {
    pub values: Vec<Flag>,
}
#[derive(Debug)]
pub struct Flag {
    pub value: u32,
    pub description: Interned,
    pub identifier: Interned,
}
#[derive(Debug)]
pub struct Message {
    pub name: Interned,
    pub members: Vec<Member>,
}
#[derive(Debug)]
pub struct Member {
    pub description: Interned,
    pub identifier: Interned,
    pub type_: Type,
}
pub struct FieldId(Cell<c_int>);

#[derive(Debug)]
pub enum Type {
    Array(ArrayType),
    BeUint16(SimpleType),
    Boolean(SimpleType),
    Data(SimpleType),
    Enum(EnumType),
    Flags(FlagsType),
    Int32(Int32Type),
    Int32String(SimpleType),
    Optional(OptionalType),
    PackedAddresses,
    Rest(SimpleType),
    ServerinfoClient,
    Sha256(SimpleType),
    SnapshotObject,
    String(StringType),
    Tick(SimpleType),
    TuneParam(SimpleType),
    Uint8(SimpleType),
    Uuid(SimpleType),
}
#[derive(Debug, Default)]
pub struct SimpleType {
    pub id: FieldId,
}
#[derive(Debug)]
pub struct ArrayType {
    pub count: i32,
    pub member_type: Box<Type>,
}
#[derive(Debug)]
pub struct EnumType {
    pub id: FieldId,
    pub tree: FieldId,
    pub id_raw: FieldId,
    pub enum_: Rc<Enumeration>,
}
#[derive(Debug)]
pub struct FlagsType {
    pub id: FieldId,
    pub tree: FieldId,
    pub id_flags: Vec<FieldId>,
    pub flags: Rc<Flags>,
}
#[derive(Debug)]
pub struct Int32Type {
    pub id: FieldId,
    pub min: Option<i32>,
    pub max: Option<i32>,
}
#[derive(Debug)]
pub struct OptionalType {
    pub inner: Box<Type>,
}
#[derive(Debug)]
pub struct StringType {
    pub id: FieldId,
    pub disallow_cc: bool,
}

#[derive(Default)]
struct Context {
    game_enumerations: HashMap<Interned, Rc<Enumeration>>,
    game_flags: HashMap<Interned, Rc<Flags>>,
}

impl Default for FieldId {
    fn default() -> FieldId {
        FieldId(Cell::new(-1))
    }
}
impl fmt::Debug for FieldId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.get().fmt(f)
    }
}
impl FieldId {
    pub fn as_ptr(&self) -> *mut c_int {
        self.0.as_ptr()
    }
    pub fn get(&self) -> c_int {
        let result = self.0.get();
        assert!(result != -1);
        result
    }
}

impl Default for Spec {
    fn default() -> Spec {
        Spec {
            prefix: intern_static_with_nul("\0"),
            game_messages: Default::default(),
            system_messages: Default::default(),
            connless_messages: Default::default(),
            tree_msg: Default::default(),
            id_msg: Default::default(),
            id_msg_system: Default::default(),
            id_msg_id_raw: Default::default(),
            id_msg_id_ex: Default::default(),
            id_connless_id_raw: Default::default(),
        }
    }
}

const PERCENT_S: &'static [u8] = b"%s\0";
const PS: *const c_char = PERCENT_S.as_ptr() as *const _;

fn load_gamenet_spec(s: &str) -> anyhow::Result<gamenet_spec::Spec> {
    Ok(serde_json::from_str(s).context("failed to parse gamenet spec")?)
}

impl Spec {
    pub fn load(prefix: Interned, s: &str) -> anyhow::Result<Spec> {
        let spec = load_gamenet_spec(s)?;
        let mut game_enumerations = HashMap::new();
        for enum_ in spec.game_enumerations {
            let enum_name = enum_.name.isnake();
            let converted = Enumeration::from_gamenet(enum_)
                .with_context(|| format!("failed to parse enumeration {}", enum_name))?;
            if game_enumerations.insert(enum_name, Rc::new(converted)).is_some() {
                bail!("duplicate game enumeration {}", enum_name);
            }
        }
        let mut game_flags = HashMap::new();
        for flags in spec.game_flags {
            let flags_name = flags.name.isnake();
            let converted = Flags::from_gamenet(flags)
                .with_context(|| format!("failed to parse flags {}", flags_name))?;
            if game_flags.insert(flags_name, Rc::new(converted)).is_some() {
                bail!("duplicate game flags {}", flags_name);
            }
        }
        let context = Context {
            game_enumerations,
            game_flags,
        };
        let mut game_messages = HashMap::new();
        for msg in spec.game_messages {
            let msg_id = msg.id;
            let converted = Message::from_gamenet(&context, prefix, false, msg)
                .with_context(|| format!("failed to parse game message id {}", msg_id))?;
            if game_messages.insert(msg_id, converted).is_some() {
                bail!("duplicate game message id {}", msg_id);
            }
        }
        let mut system_messages = HashMap::new();
        for msg in spec.system_messages {
            let msg_id = msg.id;
            let converted = Message::from_gamenet(&context, prefix, true, msg)
                .with_context(|| format!("failed to parse system message id {}", msg_id))?;
            if system_messages.insert(msg_id, converted).is_some() {
                bail!("duplicate system message id {}", msg_id);
            }
        }
        let mut connless_messages = HashMap::new();
        for msg in spec.connless_messages {
            let msg_id = msg.id;
            let converted = Message::from_gamenet_connless(&context, prefix, msg)
                .with_context(|| format!("failed to parse connless message id {}", AlmostString::new(&msg_id)))?;
            if connless_messages.insert(msg_id, converted).is_some() {
                bail!("duplicate connless message id {:?}", AlmostString::new(&msg_id));
            }
        }
        Ok(Spec {
            prefix,
            game_messages,
            system_messages,
            connless_messages,
            ..Default::default()
        })
    }
    pub fn field_register_info<FH, FT>(&self, h: &mut FH, t: &mut FT) where
        FH: FnMut(sys::hf_register_info),
        FT: FnMut(*mut c_int),
    {
        fn field(
            id: &FieldId,
            type_: sys::ftenum,
            description: &'static str,
            name: Interned,
        ) -> sys::hf_register_info {
            sys::hf_register_info {
                p_id: id.as_ptr(),
                hfinfo: sys::_header_field_info {
                    name: c(description),
                    abbrev: name.c(),
                    type_: type_,
                    display: if type_ == sys::FT_INT32 {
                        sys::BASE_DEC as c_int
                    } else {
                        HFRI_DEFAULT.display
                    },
                    ..HFRI_DEFAULT
                },
            }
        }

        h(field(&self.id_msg_system, sys::FT_BOOLEAN, "System\0", intern(&format!("{}.msg.system", self.prefix))));
        t(self.tree_msg.as_ptr());
        h(field(&self.id_msg, sys::FT_STRING, "Message\0", intern(&format!("{}.msg", self.prefix))));
        h(field(&self.id_msg_id_raw, sys::FT_INT32, "Raw message ID\0", intern(&format!("{}.msg.id_raw", self.prefix))));
        h(field(&self.id_msg_id_ex, sys::FT_GUID, "Extended message ID\0", intern(&format!("{}.msg.id_ex", self.prefix))));
        h(field(&self.id_connless_id_raw, sys::FT_STRING, "Raw connless message ID\0", intern(&format!("{}.msg.id_raw", self.prefix))));
        for msg in self.game_messages.values() {
            msg.field_register_info(h, t);
        }
        for msg in self.system_messages.values() {
            msg.field_register_info(h, t);
        }
        for msg in self.connless_messages.values() {
            msg.field_register_info(h, t);
        }
    }
    pub unsafe fn dissect<'a>(
        &self,
        tree: *mut sys::proto_tree,
        tvb: *mut sys::tvbuff_t,
        p: &mut Unpacker<'a>,
        summary: &mut dyn FnMut(&str),
    ) {
        let mut buffer: ArrayVec<[u8; 1024]> = ArrayVec::new();
        macro_rules! bformat {
            ($fmt:expr, $($args:tt)*) => {{
                buffer.clear();
                write!(buffer, $fmt, $($args)*).unwrap();
                buffer.push(0);
                CStr::from_bytes_with_nul(&buffer).unwrap().as_ptr()
            }};
        }

        let original_data = p.as_slice();

        let msg_pos = p.num_bytes_read();
        let raw_msg_pos = msg_pos;
        let raw_msg = unwrap_or!(p.read_int(&mut Ignore).ok(), return);
        let raw_msg_len = p.num_bytes_read() - raw_msg_pos;

        let system = (raw_msg & 1) != 0;
        let raw_msg = raw_msg >> 1;
        let msg_ex_pos = p.num_bytes_read();
        let msg = if raw_msg != 0 {
            MessageId::Ordinal(raw_msg)
        } else {
            MessageId::Uuid(unwrap_or!(p.read_uuid().ok(), return))
        };
        let msg_ex_len = p.num_bytes_read() - msg_ex_pos;
        let msg_len = p.num_bytes_read() - msg_pos;

        let msg_desc = if system {
            self.system_messages.get(&msg)
        } else {
            self.game_messages.get(&msg)
        };

        let mut msg_str_buf = None;
        let msg_str = msg_desc.map(|m| m.name.as_str_with_nul())
            .unwrap_or_else(|| {
                msg_str_buf = Some(format!("{}.{}\0",
                    if system { "sys" } else { "game" },
                    msg,
                ));
                msg_str_buf.as_ref().unwrap()
            });
        summary(&msg_str[..msg_str.len() - 1]);
        let id_field = sys::proto_tree_add_string_format(
            tree,
            self.id_msg.get(),
            tvb,
            msg_pos.assert_i32(),
            msg_len.assert_i32(),
            CStr::from_bytes_with_nul(msg_str.as_bytes()).unwrap().as_ptr(),
            PS,
            if let Some(d) = msg_desc {
                bformat!("Message: {}", d.name)
            } else {
                bformat!("Message: [unknown] ({})",
                    &msg_str[..msg_str.len() - 1],
                )
            },
        );
        let id_tree = sys::proto_item_add_subtree(id_field, self.tree_msg.get());
        sys::proto_tree_add_boolean_format(
            id_tree,
            self.id_msg_system.get(),
            tvb,
            raw_msg_pos.assert_i32(),
            1,
            system as c_uint,
            PS,
            bformat!("{} = {}",
                Bitfield::new(&original_data[..1], 0b0000_0001),
                if system { "System message" } else { "Game message" },
            ),
        );
        sys::proto_tree_add_int_format(
            id_tree,
            self.id_msg_id_raw.get(),
            tvb,
            raw_msg_pos.assert_i32(),
            raw_msg_len.assert_i32(),
            raw_msg,
            PS,
            bformat!("Raw message ID: {}", raw_msg),
        );
        if let MessageId::Uuid(u) = msg {
            sys::proto_tree_add_guid(
                id_tree,
                self.id_msg_id_ex.get(),
                tvb,
                msg_ex_pos.assert_i32(),
                msg_ex_len.assert_i32(),
                &to_guid(u),
            );
        }
        if let Some(d) = msg_desc {
            let _ = d.dissect(tree, tvb, p);
        }
    }
    pub unsafe fn dissect_connless<'a>(
        &self,
        tree: *mut sys::proto_tree,
        tvb: *mut sys::tvbuff_t,
        p: &mut Unpacker<'a>,
        summary: &mut dyn FnMut(&str),
    ) {
        let mut buffer: ArrayVec<[u8; 1024]> = ArrayVec::new();
        macro_rules! bformat {
            ($fmt:expr, $($args:tt)*) => {{
                buffer.clear();
                write!(buffer, $fmt, $($args)*).unwrap();
                buffer.push(0);
                CStr::from_bytes_with_nul(&buffer).unwrap().as_ptr()
            }};
        }

        let msg_pos = p.num_bytes_read();
        let msg = unwrap_or!(p.read_raw(8).ok(), return);
        let msg_len = p.num_bytes_read() - msg_pos;

        let msg = [
            msg[0], msg[1], msg[2], msg[3],
            msg[4], msg[5], msg[6], msg[7],
        ];
        let msg_desc = self.connless_messages.get(&msg);

        let msg_str_buf;
        let msg_str = match msg_desc {
            Some(m) => m.name.as_str_with_nul(),
            None => {
                if &msg[0..4] != b"\xff\xff\xff\xff" {
                    return;
                }
                msg_str_buf = format!("connless.{}\0",
                    AlmostString::new(&msg[4..8]),
                );
                &msg_str_buf
            }
        };
        summary(&msg_str[..msg_str.len() - 1]);
        let id_field = sys::proto_tree_add_string_format(
            tree,
            self.id_msg.get(),
            tvb,
            msg_pos.assert_i32(),
            msg_len.assert_i32(),
            CStr::from_bytes_with_nul(msg_str.as_bytes()).unwrap().as_ptr(),
            PS,
            if let Some(d) = msg_desc {
                bformat!("Message: {}", d.name)
            } else {
                bformat!("Message: [unknown] ({})",
                    &msg_str[..msg_str.len() - 1],
                )
            },
        );
        let id_tree = sys::proto_item_add_subtree(id_field, self.tree_msg.get());
        if let Ok(msg_raw) = CString::new(&msg[4..8]) {
            sys::proto_tree_add_string_format(
                id_tree,
                self.id_connless_id_raw.get(),
                tvb,
                msg_pos.assert_i32() + 4,
                msg_len.assert_i32() - 4,
                msg_raw.as_ptr(),
                PS,
                bformat!("Raw message ID: {:?}", &AlmostString::new(&msg[4..8])),
            );
        }
        if let Some(d) = msg_desc {
            let _ = d.dissect(tree, tvb, p);
        }
    }
}
impl Enumeration {
    fn from_gamenet(e: gamenet_spec::Enumeration) -> anyhow::Result<Enumeration> {
        let mut values = HashMap::new();
        let mut names = HashSet::new();
        for pair in e.values {
            let name = pair.name.isnake();
            if values.insert(pair.value, name).is_some() {
                bail!("duplicate game enumeration id {}", name);
            }
            if !names.insert(name) {
                bail!("duplicate game enumeration name {}", name);
            }
        }
        Ok(Enumeration {
            values,
        })
    }
}
impl Flags {
    fn from_gamenet(e: gamenet_spec::Flags) -> anyhow::Result<Flags> {
        let mut seen_names = HashSet::new();
        let mut seen_values = HashSet::new();
        let mut values = Vec::new();
        for pair in e.values {
            let name = pair.name.isnake();
            if !seen_names.insert(name) {
                bail!("duplicate game flag name {}", name);
            }
            if !seen_values.insert(pair.value) {
                bail!("duplicate game flag value {}", pair.value);
            }
            if pair.value.count_ones() != 1 {
                bail!("game flag value has more than one bit set to 1: {}", pair.value);
            }
            values.push(Flag {
                value: pair.value,
                description: pair.name.idesc(),
                identifier: name,
            });
        }
        Ok(Flags {
            values,
        })
    }
}
impl Message {
    fn from_gamenet(context: &Context, prefix: Interned, system: bool, m: gamenet_spec::Message) -> anyhow::Result<Message> {
        let sys_prefix = if system { "sys" } else { "game" };
        let name = intern(&format!("{}.{}", sys_prefix, m.name.snake()));
        let prefix = intern(&format!("{}.{}", prefix, name));
        Ok(Message {
            name,
            members: m.members.into_iter().map(
                |member| Member::from_gamenet(context, prefix, member)
            ).collect::<Result<_, _>>()?,
        })
    }
    fn from_gamenet_connless(context: &Context, prefix: Interned, m: gamenet_spec::ConnlessMessage) -> anyhow::Result<Message> {
        let name = intern(&format!("connless.{}", m.name.snake()));
        let prefix = intern(&format!("{}.{}", prefix, name));
        Ok(Message {
            name,
            members: m.members.into_iter().map(
                |member| Member::from_gamenet(context, prefix, member)
            ).collect::<Result<_, _>>()?,
        })
    }
    pub fn field_register_info<FH, FT>(&self, h: &mut FH, t: &mut FT) where
        FH: FnMut(sys::hf_register_info),
        FT: FnMut(*mut c_int),
    {
        for m in &self.members {
            m.field_register_info(h, t);
        }
    }
    pub unsafe fn dissect<'a>(
        &self,
        tree: *mut sys::proto_tree,
        tvb: *mut sys::tvbuff_t,
        p: &mut Unpacker<'a>,
    ) -> Result<(), ()> {
        for m in &self.members {
            m.type_.dissect(m.description, tree, tvb, p)?;
        }
        Ok(())
    }
}
impl Member {
    fn from_gamenet(context: &Context, prefix: Interned, m: gamenet_spec::Member) -> anyhow::Result<Member> {
        Ok(Member {
            description: intern(&m.name.desc()),
            identifier: intern(&format!("{}.{}", prefix, m.name.snake())),
            type_: Type::from_gamenet(context, m.type_)?,
        })
    }
    pub fn field_register_info<FH, FT>(&self, h: &mut FH, t: &mut FT) where
        FH: FnMut(sys::hf_register_info),
        FT: FnMut(*mut c_int),
    {
        self.type_.field_register_info(h, t, self.description, self.identifier);
    }
}
impl Type {
    fn from_gamenet(context: &Context, t: gamenet_spec::Type) -> anyhow::Result<Type> {
        use gamenet_spec::Type::*;
        Ok(match t {
            Array(i) => Type::Array(ArrayType {
                count: i.count,
                member_type: Box::new(Type::from_gamenet(context, *i.member_type)?),
            }),
            BeUint16 => Type::BeUint16(Default::default()),
            Boolean => Type::Boolean(Default::default()),
            Data => Type::Data(Default::default()),
            Enum(i) => Type::Enum(EnumType {
                id: Default::default(),
                tree: Default::default(),
                id_raw: Default::default(),
                enum_: context.game_enumerations.get(&i.enum_.isnake())
                    .ok_or_else(|| anyhow!("unknown enumeration {}", i.enum_.snake()))?
                    .clone(),
            }),
            Flags(i) => {
                let flags = context.game_flags.get(&i.flags.isnake())
                    .ok_or_else(|| anyhow!("unknown flags {}", i.flags.snake()))?
                    .clone();
                Type::Flags(FlagsType {
                    id: Default::default(),
                    tree: Default::default(),
                    id_flags: (0..flags.values.len()).map(|_| Default::default()).collect(),
                    flags,
                })
            },
            Int32(i) => Type::Int32(Int32Type {
                id: Default::default(),
                min: i.min,
                max: i.max,
            }),
            Int32String => Type::Int32String(Default::default()),
            Optional(i) => Type::Optional(OptionalType {
                inner: Box::new(Type::from_gamenet(context, *i.inner)?),
            }),
            PackedAddresses => Type::PackedAddresses,
            Rest => Type::Rest(Default::default()),
            ServerinfoClient => Type::ServerinfoClient,
            Sha256 => Type::Sha256(Default::default()),
            SnapshotObject(..) => Type::SnapshotObject,
            String(i) => Type::String(StringType {
                id: Default::default(),
                disallow_cc: i.disallow_cc,
            }),
            Tick => Type::Tick(Default::default()),
            TuneParam => Type::TuneParam(Default::default()),
            Uint8 => Type::Uint8(Default::default()),
            Uuid => Type::Uuid(Default::default()),
        })
    }
}
impl Type {
    pub fn field_register_info<FH, FT>(
        &self, h: &mut FH, t: &mut FT, desc: Interned, identifier: Interned
    ) where
        FH: FnMut(sys::hf_register_info),
        FT: FnMut(*mut c_int),
    {
        use self::Type::*;
        let (type_, hf_id) = match self {
            Array(i) =>
                return i.member_type.field_register_info(h, t, desc, identifier),
            BeUint16(i) => (sys::FT_UINT16, i.id.as_ptr()),
            Boolean(i) => (sys::FT_BOOLEAN, i.id.as_ptr()),
            Data(i) => (sys::FT_BYTES, i.id.as_ptr()),
            Enum(i) => {
                t(i.tree.as_ptr());
                h(sys::hf_register_info {
                    p_id: i.id.as_ptr(),
                    hfinfo: sys::_header_field_info {
                        name: desc.c(),
                        abbrev: identifier.c(),
                        type_: sys::FT_STRING,
                        ..HFRI_DEFAULT
                    },
                });
                h(sys::hf_register_info {
                    p_id: i.id_raw.as_ptr(),
                    hfinfo: sys::_header_field_info {
                        name: intern(&format!("{} (raw)", desc)).c(),
                        abbrev: intern(&format!("{}.raw", identifier)).c(),
                        type_: sys::FT_INT32,
                        display: sys::BASE_DEC as c_int,
                        ..HFRI_DEFAULT
                    },
                });
                return;
            },
            Flags(i) => {
                t(i.tree.as_ptr());
                h(sys::hf_register_info {
                    p_id: i.id.as_ptr(),
                    hfinfo: sys::_header_field_info {
                        name: desc.c(),
                        abbrev: identifier.c(),
                        type_: sys::FT_UINT32,
                        display: sys::BASE_HEX as c_int,
                        ..HFRI_DEFAULT
                    },
                });
                for (idx, id) in i.id_flags.iter().enumerate() {
                    h(sys::hf_register_info {
                        p_id: id.as_ptr(),
                        hfinfo: sys::_header_field_info {
                            name: i.flags.values[idx].description.c(),
                            abbrev: intern(&format!("{}.{}", identifier, i.flags.values[idx].identifier)).c(),
                            type_: sys::FT_BOOLEAN,
                            ..HFRI_DEFAULT
                        },
                    });
                }
                return;
            },
            Int32(i) => (sys::FT_INT32, i.id.as_ptr()),
            Int32String(i) => (sys::FT_INT32, i.id.as_ptr()),
            Optional(i) =>
                return i.inner.field_register_info(h, t, desc, identifier),
            PackedAddresses => return,
            Rest(i) => (sys::FT_BYTES, i.id.as_ptr()),
            ServerinfoClient => return,
            Sha256(i) => (sys::FT_STRING, i.id.as_ptr()),
            SnapshotObject => return,
            String(i) => (sys::FT_STRINGZ, i.id.as_ptr()),
            Tick(i) => (sys::FT_INT32, i.id.as_ptr()),
            TuneParam(i) => (sys::FT_FLOAT, i.id.as_ptr()),
            Uint8(i) => (sys::FT_UINT8, i.id.as_ptr()),
            Uuid(i) => (sys::FT_GUID, i.id.as_ptr()),
        };
        let display = match type_ {
            sys::FT_INT32 | sys::FT_UINT8 | sys::FT_UINT16 => sys::BASE_DEC as c_int,
            _ => HFRI_DEFAULT.display,
        };
        h(sys::hf_register_info {
            p_id: hf_id,
            hfinfo: sys::_header_field_info {
                name: desc.c(),
                abbrev: identifier.c(),
                type_,
                display,
                ..HFRI_DEFAULT
            },
        });
    }
    pub unsafe fn dissect<'a>(
        &self,
        desc: Interned,
        tree: *mut sys::proto_tree,
        tvb: *mut sys::tvbuff_t,
        p: &mut Unpacker<'a>,
    ) -> Result<(), ()> {
        let mut buffer: ArrayVec<[u8; 1024]> = ArrayVec::new();
        macro_rules! bformat {
            ($fmt:expr, $($args:tt)*) => {{
                buffer.clear();
                write!(buffer, $fmt, $($args)*).unwrap();
                buffer.push(0);
                CStr::from_bytes_with_nul(&buffer).unwrap().as_ptr()
            }};
        }

        let pos = p.num_bytes_read();
        use self::Type::*;
        match self {
            Array(i) => {
                for _ in 0..i.count {
                    i.member_type.dissect(desc, tree, tvb, p)?;
                }
            },
            BeUint16(i) => {
                let v = p.read_raw(2).map_err(|_| ())?;
                let v = BeU16::from_bytes(&[v[0], v[1]]).to_u16();
                sys::proto_tree_add_uint_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_uint,
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
            Boolean(i) => {
                let v = p.read_int(&mut Ignore).map_err(|_| ())? != 0;
                sys::proto_tree_add_boolean_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_uint,
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
            Data(i) => {
                let v = p.read_data(&mut Ignore).map_err(|_| ())?;
                let ti = sys::proto_tree_add_bytes_with_length(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v.as_ptr(),
                    v.len().assert_i32(),
                );
                sys::proto_item_set_text(ti, PS, bformat!("{} ({})",
                    desc,
                    NumBytes::new(v.len()),
                ));
            },
            Enum(i) => {
                let v = p.read_int(&mut Ignore).map_err(|_| ())?;
                let name = i.enum_.values.get(&v);
                let buffer;
                let name_c = if let Some(name) = name {
                    name.c()
                } else {
                    buffer = CString::new(format!("{}", v)).unwrap();
                    buffer.as_ptr()
                };
                let id_field = sys::proto_tree_add_string_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    name_c,
                    PS,
                    if let Some(name) = name {
                        bformat!("{}: {}", desc, name)
                    } else {
                        bformat!("{}: [unknown] ({})", desc, v)
                    },
                );
                let id_subtree = sys::proto_item_add_subtree(id_field, i.tree.get());
                sys::proto_tree_add_int_format(
                    id_subtree,
                    i.id_raw.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_int,
                    PS,
                    bformat!("Raw: {}", v),
                );
            },
            Flags(i) => {
                let v = p.read_int(&mut Ignore).map_err(|_| ())? as u32;
                let mut flag_names: CommaSeparated<[u8; 256]> = CommaSeparated::new();
                for flag in &i.flags.values {
                    if v & flag.value != 0 {
                        flag_names.add(flag.identifier.as_str());
                    }
                }
                let id_field = sys::proto_tree_add_uint_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_uint,
                    PS,
                    bformat!("{}: {} (0x{:x})", desc, flag_names.or("none"), v),
                );
                let id_subtree = sys::proto_item_add_subtree(id_field, i.tree.get());
                let num_unused_bytes = i.flags.values.iter()
                    .map(|v| v.value.leading_zeros())
                    .min()
                    .unwrap_or(0)
                    .usize()
                    / 8;
                let v_bytes = &v.to_be_bytes()[num_unused_bytes..];
                for (idx, id) in i.id_flags.iter().enumerate() {
                    let flag = i.flags.values[idx].value;
                    sys::proto_tree_add_boolean_format(
                        id_subtree,
                        id.get(),
                        tvb,
                        pos.assert_i32(),
                        (p.num_bytes_read() - pos).assert_i32(),
                        (v & flag != 0) as c_uint,
                        PS,
                        bformat!("{} = {} {}",
                            Bitfield::new(v_bytes, flag.u64()),
                            if v & flag != 0 { "   " } else { "NOT" },
                            i.flags.values[idx].identifier,
                        ),
                    );
                }
            },
            Int32(i) => {
                let v = p.read_int(&mut Ignore).map_err(|_| ())?;
                sys::proto_tree_add_int_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_int,
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
            Int32String(i) => {
                let v = p.read_string().map_err(|_| ())?;
                let v = str::from_utf8(v).map_err(|_| ())?;
                let v: i32 = v.parse().map_err(|_| ())?;
                sys::proto_tree_add_int_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_int,
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
            Optional(i) => {
                let _ = i.inner.dissect(desc, tree, tvb, p);
                return Ok(());
            },
            PackedAddresses => return Err(()),
            Rest(i) => {
                let v = p.read_rest().map_err(|_| ())?;
                let ti = sys::proto_tree_add_bytes_with_length(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v.as_ptr(),
                    v.len().assert_i32(),
                );
                sys::proto_item_set_text(ti, PS, bformat!("{} ({})",
                    desc,
                    NumBytes::new(v.len()),
                ));
            },
            ServerinfoClient => return Err(()),
            Sha256(i) => {
                let size = mem::size_of::<digest::Sha256>();
                let v = p.read_raw(size).map_err(|_| ())?;
                let v = digest::Sha256::from_slice(v).unwrap();
                let cstring_v = CString::new(format!("{}", v)).unwrap();
                sys::proto_tree_add_string_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    cstring_v.as_ptr(),
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            }
            SnapshotObject => return Err(()),
            String(i) => {
                let v = p.read_string().map_err(|_| ())?;
                let cstring_v = CString::new(v).unwrap();
                sys::proto_tree_add_string_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    cstring_v.as_ptr(),
                    PS,
                    bformat!("{}: {:?}", desc, AlmostString::new(v)),
                );
            },
            Tick(i) => {
                let v = p.read_int(&mut Ignore).map_err(|_| ())?;
                sys::proto_tree_add_int_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_int,
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
            TuneParam(i) => {
                let raw_v = p.read_int(&mut Ignore).map_err(|_| ())?;
                let v = raw_v as f32 / 100.0;
                sys::proto_tree_add_float_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v,
                    PS,
                    bformat!("{}: {} (raw: {})", desc, v, raw_v),
                );
            },
            Uint8(i) => {
                let v = p.read_raw(1).map_err(|_| ())?[0];
                sys::proto_tree_add_uint_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    v as c_uint,
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
            Uuid(i) => {
                let v = p.read_uuid().map_err(|_| ())?;
                sys::proto_tree_add_guid_format(
                    tree,
                    i.id.get(),
                    tvb,
                    pos.assert_i32(),
                    (p.num_bytes_read() - pos).assert_i32(),
                    &to_guid(v),
                    PS,
                    bformat!("{}: {}", desc, v),
                );
            },
        };
        Ok(())
    }
}
