from collections import namedtuple
import uuid
import threading

def title(c):
    return "".join(p.title() for p in c)

SNAKE_REPLACEMENTS = {
    ("self",): "self_",
    ("type",): "type_",
}
def snake(c):
    if isinstance(c, tuple) and c in SNAKE_REPLACEMENTS:
        return SNAKE_REPLACEMENTS[c]
    return "_".join(c)

def caps(c):
    return "_".join(p.upper() for p in c)

def canonicalize(s):
    if isinstance(s, tuple):
        return s
    if s.isdigit():
        s = "v{}".format(s)
    if s.isupper() or s.islower():
        return tuple(p.lower() for p in s.split("_"))
    PREFIXES=sorted(["m_", "m_a", "m_aa", "m_ap", "m_p"], key=len, reverse=True)
    for prefix in PREFIXES:
        if s.startswith(prefix):
            s = s[len(prefix):]
    s = s.replace("ID", "Id")
    s = s.replace("DDNet", "Ddnet")
    s = s.replace("DDRace", "Ddrace")
    result = []
    first = True
    for c in s:
        if not first and c.isupper():
            result.append("_")
        first = False
        if c == "_":
            continue
        result.append(c.lower())
    return tuple("".join(result).split("_"))

class ProtocolSpecError(ValueError):
    pass

PROTOCOL_PARTS=[
    "constants",
    "game_enumerations",
    "game_flags",
    "game_messages",
    "snapshot_objects",
    "system_messages",
    "connless_messages",
]
class ProtocolSpec(namedtuple("ProtocolSpec", PROTOCOL_PARTS)):
    pass

def load_protocol_spec(json_obj):
    constants = [Constant.deserialize(e) for e in json_obj["constants"]]
    game_enumerations = [Enum.deserialize(e) for e in json_obj["game_enumerations"]]
    game_flags = [Flags.deserialize(e) for e in json_obj["game_flags"]]
    game_messages = [NetMessage.deserialize(e) for e in json_obj["game_messages"]]
    snapshot_objects = [NetObject.deserialize(e) for e in json_obj["snapshot_objects"]]
    system_messages = [NetMessage.deserialize(e) for e in json_obj["system_messages"]]
    connless_messages = [NetConnless.deserialize(e) for e in json_obj["connless_messages"]]
    structs = {o.name: o for o in snapshot_objects}
    for o in snapshot_objects:
        o.structs = structs
    return ProtocolSpec(
        constants,
        game_enumerations,
        game_flags,
        game_messages,
        snapshot_objects,
        system_messages,
        connless_messages,
    )

def deserialize_member(json_obj):
    name = ()
    if "name" in json_obj:
        name = tuple(json_obj["name"])
        json_obj = json_obj["type"]
    kind = json_obj["kind"]
    if kind not in MEMBER_TYPES_MAPPING:
        raise ProtocolSpecError("unknown member kind {!r}".format(kind))
    kwargs = {}
    if "default" in json_obj:
        kwargs["default"] = json_obj["default"]
    return MEMBER_TYPES_MAPPING[kind].deserialize(name, json_obj, **kwargs)

DDNET_EX_UUID="e05ddaaa-c4e6-4cfb-b642-5d48e80c0029"
def uuid_v3(namespace, name):
    return str(uuid.uuid3(uuid.UUID(namespace), name))

class NameValues:
    def __init__(self, name, values, ex=None, teehistorian=True):
        names = name.split(':')
        if not 1 <= len(names) <= 2:
            raise ValueError("invalid name format")
        self.name = canonicalize(names[0])
        self.super = canonicalize(names[1]) if len(names) == 2 else None
        self.values = values
        self.ex = ex
        self.attributes = set()
        if not teehistorian:
            self.attributes.add("nonteehistoric")
    def init(self, index, consts, enums, structs):
        if self.ex is None:
            self.index = index
        else:
            self.index = uuid_v3(DDNET_EX_UUID, self.ex)
        self.enums = enums
        self.structs = structs
    def serialize(self):
        result = {}
        result["id"] = self.index
        if self.ex is not None:
            result["id_from"] = {
                "algorithm": "uuid_v3",
                "namespace": DDNET_EX_UUID,
                "name": self.ex,
            }
        result["name"] = self.name
        if self.super is not None:
            result["super"] = self.super
        result["members"] = [v.serialize() for v in self.values]
        result["attributes"] = sorted(self.attributes)
        return result
    @classmethod
    def deserialize(cls, json_obj):
        if "super" in json_obj:
            name = "{}:{}".format(snake(json_obj["name"]), snake(json_obj["super"]))
        else:
            name = snake(json_obj["name"])
        result = cls(name, [deserialize_member(m) for m in json_obj["members"]])
        result.index = json_obj["id"]
        if "id_from" in json_obj:
            if json_obj["id_from"]["algorithm"] == "uuid_v3" and json_obj["id_from"]["namespace"] == DDNET_EX_UUID:
                result.ex = json_obj["id_from"]["name"]
        result.attributes = set(json_obj["attributes"])
        return result

class Emit:
    def __init__(self):
        self.cur_indent = 0
        self.lines = []
        self.imports = set()
        self.previous_emits = []
    def __enter__(self):
        self.previous_emits.append(_emit_get())
        _emit_set(self)
    def __exit__(self, exc_type, exc_value, traceback):
        if _emit_get() != self:
            raise RuntimeError("unexpected value for current emit")
        _emit_set(self.previous_emits.pop())
    def indent(self, level=1):
        class Indent:
            def __init__(self, emit, level):
                self.emit = emit
                self.level = level
            def __enter__(self):
                self.emit.cur_indent += self.level
            def __exit__(self, exc_type, exc_value, traceback):
                self.emit.cur_indent -= self.level
        return Indent(self, level)
    def print(self, string=""):
        self.lines += ["    " * self.cur_indent + l for l in (string + "\n").splitlines()]
    def import_(self, *args):
        self.imports.update(args)
    def get(self):
        imports = []
        if self.imports:
            for i in sorted(self.imports):
                imports.append("use {};".format(i))
            imports.append("")
        return "\n".join(imports + self.lines + [""])
    def dump(self):
        _print(self.get(), end="")

thread_local = threading.local()
thread_local.emit = object()

def _emit_set(emit):
    thread_local.emit = emit

def _emit_get():
    return thread_local.emit

_print = print
def print(*args):
    return _emit_get().print(*args)

def import_(*args):
    return _emit_get().import_(*args)

def indent(*args):
    return _emit_get().indent(*args)

def emit_header_enums():
    pass

def emit_header_snap_obj():
    print("""\
pub use gamenet_common::snap_obj::Tick;
pub use gamenet_common::snap_obj::TypeId;
""")

def emit_header_msg_system():
    import_(
        "buffer::CapacityError",
        "error::Error",
        "packer::Packer",
        "packer::Unpacker",
        "packer::Warning",
        "packer::with_packer",
        "super::SystemOrGame",
        "warn::Warn",
    )
    print("""\
impl<'a> System<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<System<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::System(msg_id) = SystemOrGame::decode_id(warn, p)? {
            System::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        with_packer(&mut p, |p| SystemOrGame::System(self.msg_id()).encode_id(p))?;
        with_packer(&mut p, |p| self.encode_msg(p))?;
        Ok(p.written())
    }
}
""")

def emit_header_msg_game():
    import_(
        "buffer::CapacityError",
        "error::Error",
        "packer::Packer",
        "packer::Unpacker",
        "packer::Warning",
        "packer::with_packer",
        "super::SystemOrGame",
        "warn::Warn",
    )
    print("""\
pub use gamenet_common::msg::TuneParam;

impl<'a> Game<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<Game<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::Game(msg_id) = SystemOrGame::decode_id(warn, p)? {
            Game::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        with_packer(&mut p, |p| SystemOrGame::Game(self.msg_id()).encode_id(p))?;
        with_packer(&mut p, |p| self.encode_msg(p))?;
        Ok(p.written())
    }
}
""")

def emit_header_msg_connless(structs):
    import_(
        "buffer::CapacityError",
        "common::pretty",
        "error::Error",
        "packer::Packer",
        "packer::Unpacker",
        "packer::Warning",
        "packer::with_packer",
        "std::fmt",
        "gamenet_common::msg::string_from_int",
        "warn::Warn",
    )
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    print("""\
impl{l} Connless{l} {{
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker{l}) -> Result<Connless{l}, Error> {{
        let id = _p.read_raw(8)?;
        let connless_id = [id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]];
        Connless::decode_connless(warn, connless_id, _p)
    }}
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {{
        p.write_raw(&self.connless_id())?;
        with_packer(&mut p, |p| self.encode_connless(p))?;
        Ok(p.written())
    }}
}}

pub struct Client<'a> {{
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}}

impl<'a> Client<'a> {{
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {{
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_string(&string_from_int(self.country))?;
        _p.write_string(&string_from_int(self.score))?;
        _p.write_string(&string_from_int(self.is_player))?;
        Ok(_p.written())
    }}
}}

impl<'a> fmt::Debug for Client<'a> {{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{
        f.debug_struct("Client")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("score", &self.score)
            .field("is_player", &self.is_player)
            .finish()
    }}
}}

pub const INFO_FLAG_PASSWORD: i32 = 1;
""".format(l=lifetime))

def emit_enum_def(name, structs):
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    print("#[derive(Clone, Copy)]")
    print("pub enum {}{} {{".format(title(name), lifetime))
    for s in structs:
        print("    {}({}{}),".format(title(s.name), title(s.name), s.lifetime()))
    print("}")

def emit_enum_from(name, structs):
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    for s in structs:
        print()
        print("impl{l} From<{}{}> for {}{l} {{".format(title(s.name), s.lifetime(), title(name), l=lifetime))
        print("    fn from(i: {}{}) -> {}{l} {{".format(title(s.name), s.lifetime(), title(name), l=lifetime))
        print("        {}::{}(i)".format(title(name), title(s.name)))
        print("    }")
        print("}")

def emit_enum_msg(name, structs):
    import_(
        "buffer::CapacityError",
        "error::Error",
        "packer::Packer",
        "packer::Unpacker",
        "packer::Warning",
        "std::fmt",
        "super::MessageId",
        "warn::Warn",
    )
    name = canonicalize(name)
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    emit_enum_def(name, structs)
    print()
    print("impl{l} {}{l} {{".format(title(name), l=lifetime))
    print("    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: MessageId, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(title(name), l=lifetime))
    print("        use self::MessageId::*;")
    print("        Ok(match msg_id {")
    for s in structs:
        constructor = "Ordinal" if isinstance(s.index, int) else "Uuid"
        print("            {}({}) => {}::{s}({s}::decode(warn, _p)?),".format(constructor, caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownId),".format(caps(s.name), title(name), s=title(s.name)))
    print("        })")
    print("    }")
    print("    pub fn msg_id(&self) -> MessageId {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(_) => MessageId::from({}),".format(title(name), title(s.name), caps(s.name)))
    print("        }")
    print("    }")
    print("    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.encode(p),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    print()
    print("impl{l} fmt::Debug for {}{l} {{".format(title(name), l=lifetime))
    print("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.fmt(f),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    emit_enum_from(name, structs)

def emit_enum_msg_module(name, structs):
    for s in structs:
        s.emit_consts()
    print()
    emit_enum_msg(name, structs)
    for s in structs:
        s.emit_definition()
        print()
    for s in structs:
        s.emit_impl_encode_decode()
        s.emit_maybe_default()
        s.emit_impl_debug()
        print()

def emit_enum_obj(name, structs):
    import_(
        "error::Error",
        "packer::ExcessData",
        "packer::IntUnpacker",
        "std::fmt",
        "warn::Warn",
    )
    name = canonicalize(name)
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    emit_enum_def(name, structs)
    print()
    print("impl{l} {}{l} {{".format(title(name), l=lifetime))
    print("    pub fn decode_obj<W: Warn<ExcessData>>(warn: &mut W, obj_type_id: TypeId, _p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(name), l=lifetime))
    print("        use self::TypeId::*;")
    print("        Ok(match obj_type_id {")
    for s in structs:
        constructor = "Ordinal" if isinstance(s.index, int) else "Uuid"
        print("            {}({}) => {}::{s}({s}::decode(warn, _p)?),".format(constructor, caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownId),".format(caps(s.name), title(name), s=title(s.name)))
    print("        })")
    print("    }")
    print("    pub fn obj_type_id(&self) -> TypeId {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(_) => TypeId::from({}),".format(title(name), title(s.name), caps(s.name)))
    print("        }")
    print("    }")
    print("    pub fn encode(&self) -> &[i32] {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.encode(),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    print()
    print("impl{l} fmt::Debug for {}{l} {{".format(title(name), l=lifetime))
    print("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.fmt(f),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    emit_enum_from(name, structs)

def emit_enum_obj_module(name, structs, flags):
    for f in flags:
        f.emit_definition()
        print()
    for s in structs:
        s.emit_consts()
    print()
    emit_enum_obj(name, structs)
    print()
    for s in structs:
        s.emit_definition()
        print()
    for s in structs:
        s.emit_impl_debug()
        s.emit_impl_encode_decode_int()
        if "msg_encoding" in s.attributes:
            s.emit_impl_encode_decode(suffix=True)
        print()
    emit_snap_obj_sizes(structs)

def emit_enum_connless(name, structs):
    import_(
        "buffer::CapacityError",
        "error::Error",
        "packer::Warning",
        "std::fmt",
        "warn::Warn",
    )
    name = canonicalize(name)
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    emit_enum_def(name, structs)
    print()
    print("impl{l} {}{l} {{".format(title(name), l=lifetime))
    print("    pub fn decode_connless<W: Warn<Warning>>(warn: &mut W, connless_id: [u8; 8], _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(title(name), l=lifetime))
    print("        Ok(match &connless_id {")
    for s in structs:
        print("            {} => {}::{s}({s}::decode(warn, _p)?),".format(caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownId),")
    print("        })")
    print("    }")
    print("    pub fn connless_id(&self) -> [u8; 8] {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(_) => *{},".format(title(name), title(s.name), caps(s.name)))
    print("        }")
    print("    }")
    print("    pub fn encode_connless<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.encode(p),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    print()
    print("impl{l} fmt::Debug for {}{l} {{".format(title(name), l=lifetime))
    print("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.fmt(f),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    emit_enum_from(name, structs)

def emit_enum_connless_module(name, structs):
    for s in structs:
        s.emit_consts()
    print()
    emit_enum_connless(name, structs)
    for s in structs:
        s.emit_definition()
        print()
    for s in structs:
        s.emit_impl_encode_decode()
        s.emit_impl_debug()
        print()

def emit_snap_obj_sizes(objects):
    print("pub fn obj_size(type_: u16) -> Option<u32> {")
    print("    Some(match type_ {")
    for o in objects:
        if isinstance(o.index, int):
            print("        {} => {},".format(caps(o.name), o.int_size()))
    print("        _ => return None,")
    print("    })")
    print("}")

def emit_enum_module(consts, enums):
    for c in consts:
        c.emit_definition()
    print()
    for e in enums:
        e.emit_definition()
        print()
    for e in enums:
        e.emit_impl()
        print()

def emit_cargo_toml(name):
    print("""\
[package]
name = "{}"
version = "0.0.1"
authors = ["heinrich5991 <heinrich5991@gmail.com>"]
license = "MIT/Apache-2.0"

[dependencies]
arrayvec = "0.5.2"
buffer = "0.1.9"
common = {{ path = "../../common/" }}
gamenet_common = {{ path = "../common/" }}
packer = {{ path = "../../packer/", features = ["uuid"] }}
uuid = "0.8.1"
warn = ">=0.1.1,<0.3.0"\
""".format(name))

def emit_main_lib():
    print("""\
extern crate arrayvec;
extern crate buffer;
extern crate common;
extern crate gamenet_common;
extern crate packer;
extern crate uuid;
extern crate warn;

pub mod enums;
pub mod msg;
pub mod snap_obj;

pub use gamenet_common::error;
pub use gamenet_common::error::Error;
pub use snap_obj::SnapObj;\
""")

def emit_msg_module():
    import_(
        "gamenet_common::error::Error",
        "packer::Unpacker",
        "packer::Warning",
        "warn::Warn",
    )
    print("""\
pub mod connless;
pub mod game;
pub mod system;

pub use self::connless::Connless;
pub use self::game::Game;
pub use self::system::System;

pub use gamenet_common::msg::AddrPacked;
pub use gamenet_common::msg::CLIENTS_DATA_NONE;
pub use gamenet_common::msg::ClientsData;
pub use gamenet_common::msg::MessageId;
pub use gamenet_common::msg::SystemOrGame;

struct Protocol;

impl<'a> gamenet_common::msg::Protocol<'a> for Protocol {
    type System = System<'a>;
    type Game = Game<'a>;

    fn decode_system<W>(warn: &mut W, id: MessageId, p: &mut Unpacker<'a>)
        -> Result<Self::System, Error>
        where W: Warn<Warning>
    {
        System::decode_msg(warn, id, p)
    }
    fn decode_game<W>(warn: &mut W, id: MessageId, p: &mut Unpacker<'a>)
        -> Result<Self::Game, Error>
        where W: Warn<Warning>
    {
        Game::decode_msg(warn, id, p)
    }
}

pub fn decode<'a, W>(warn: &mut W, p: &mut Unpacker<'a>)
    -> Result<SystemOrGame<System<'a>, Game<'a>>, Error>
    where W: Warn<Warning>
{
    gamenet_common::msg::decode(warn, Protocol, p)
}
""")

class Enum(NameValues):
    def __init__(self, name, values, offset=0):
        super().__init__(name, [canonicalize(v) for v in values])
        self.offset = offset
    def emit_definition(self):
        import_(
            "packer::IntOutOfRange",
        )
        for i, name in enumerate(self.values):
            print("pub const {}_{}: i32 = {};".format(caps(self.name), caps(name), i + self.offset))
        print()
        print("#[repr(i32)]")
        print("#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]")
        print("pub enum {} {{".format(title(self.name)))
        for i, name in enumerate(self.values):
            if self.offset == 0 or i != 0:
                print("    {},".format(title(name)))
            else:
                print("    {} = {},".format(title(name), self.offset))
        print("}")

    def emit_impl(self):
        print("impl {} {{".format(title(self.name)))
        print("    pub fn from_i32(i: i32) -> Result<{}, IntOutOfRange> {{".format(title(self.name)))
        print("        use self::{}::*;".format(title(self.name)))
        print("        Ok(match i {")
        for name in self.values:
            print("            {}_{} => {},".format(caps(self.name), caps(name), title(name)))
        print("            _ => return Err(IntOutOfRange),")
        print("        })")
        print("    }")
        print("    pub fn to_i32(self) -> i32 {")
        print("        use self::{}::*;".format(title(self.name)))
        print("        match self {")
        for name in self.values:
            print("            {} => {}_{},".format(title(name), caps(self.name), caps(name)))
        print("        }")
        print("    }")
        print("}")
    def serialize(self):
        return {
            "name": self.name,
            "values": [{"value": self.offset + i, "name": name} for i, name in enumerate(self.values)],
        }
    @staticmethod
    def deserialize(json_obj):
        values = sorted(json_obj["values"], key=lambda x: x["value"])
        if any(v["value"] != values[0]["value"] + i for i, v in enumerate(values)):
            raise ProtocolSpecError("Only supporting contiguous enums")
        offset = 0
        if values:
            offset = values[0]["value"]
        return Enum(
            snake(json_obj["name"]),
            [tuple(v["name"]) for v in values],
            offset=offset,
        )


class Flags(NameValues):
    def __init__(self, name, values):
        super().__init__(name, [canonicalize(v) for v in values])
    def emit_definition(self):
        for i, name in enumerate(self.values):
            print("pub const {}_{}: i32 = 1 << {};".format(caps(self.name), caps(name), i))
    def serialize(self):
        return {
            "name": self.name,
            "values": [{"value": 1 << i, "name": name} for i, name in enumerate(self.values)],
        }
    @staticmethod
    def deserialize(json_obj):
        values = sorted(json_obj["values"], key=lambda x: x["value"])
        if any(v["value"] != 1 << i for i, v in enumerate(values)):
            raise ProtocolSpecError("Only supporting contiguous flags")
        return Flags(
            snake(json_obj["name"]),
            [tuple(v["name"]) for v in values],
        )

def lifetime(members):
    return "<'a>" if any(m.contains_lifetime() for m in members) else ""

class Struct(NameValues):
    def lifetime(self):
        result = lifetime(self.values)
        if self.super:
            result = result or self.structs[self.super].lifetime()
        return result
    def init(self, index, consts, enums, structs):
        super().init(index, consts, enums, structs)

        old_members = self.values
        self.values = []
        array_type = None
        array_len = 0
        for m in old_members + [None]:
            s_name = m and snake(m.name)
            a_name = m and s_name.rstrip("0123456789")
            if (m is not None
                    and (array_type is None or snake(array_type.name) == a_name)
                    and s_name == a_name + str(array_len)):
                array_type = m
                array_type.name = canonicalize(a_name)
                array_len += 1
            else:
                if array_type is not None:
                    self.values.append(NetArray(array_type.name, array_type, array_len))
                array_type = None
                array_len = 0
                if m is not None and s_name != a_name + "0":
                    self.values.append(m)
                elif m is not None:
                    array_type = m
                    array_type.name = canonicalize(a_name)
                    array_len += 1

        if self.name == ("sv", "chat"):
            for i in range(len(self.values)):
                if (type(self.values[i]) == NetIntRange
                        and self.values[i].name == ("team",)
                        and self.values[i].min == "TEAM_SPECTATORS"
                        and self.values[i].max == "TEAM_BLUE"):
                    self.values[i] = NetBool(self.values[i].name)
        self.values = [member.update(self, consts, enums, structs) for member in self.values]

    def emit_consts(self):
        if isinstance(self.index, int):
            type_ = self.const_type
            value = self.index
        else:
            import_("uuid::Uuid")
            type_ = "Uuid"
            value = "Uuid::from_u128(0x{})".format(self.index.replace("-", "_"))
        print("pub const {}: {} = {};".format(caps(self.name), type_, value))
    def emit_definition(self):
        if self.super:
            super = self.structs[self.super]
        else:
            super = None

        if self.name != ("player", "input"):
            print("#[derive(Clone, Copy)]")
        else:
            print("#[derive(Clone, Copy, Default)]")
        if self.values or super:
            print("pub struct {}{} {{".format(title(self.name), self.lifetime()))
            if super:
                print("    pub {}: {}{},".format(snake(super.name), title(super.name), super.lifetime()))
            for member in self.values:
                print("    pub {},".format(member.definition()))
            print("}")
        else:
            print("pub struct {};".format(title(self.name)))
    def emit_impl_encode_decode(self, suffix=False):
        import_(
            "buffer::CapacityError",
            "error::Error",
            "packer::Packer",
            "packer::Unpacker",
            "packer::Warning",
            "std::fmt",
            "warn::Warn",
        )
        if suffix:
            suffix = "_msg"
        else:
            suffix = ""
        print("impl{l} {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    pub fn decode{}<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(suffix, title(self.name), l=self.lifetime()))
        if self.values:
            print("        let result = Ok({} {{".format(title(self.name)))
            with indent(3):
                for m in self.values:
                    m.emit_decode()
            print("        });")
        else:
            print("        let result = Ok({});".format(title(self.name)))
        print("        _p.finish(warn);")
        print("        result")
        print("    }")
        print("    pub fn encode{}<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {{".format(suffix, title(self.name), l=self.lifetime()))
        with indent(2):
            for m in self.values:
                m.emit_assert()
            for m in self.values:
                m.emit_encode()
        print("        Ok(_p.written())")
        print("    }")
        print("}")
    def emit_impl_debug(self):
        print("impl{l} fmt::Debug for {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
        print("        f.debug_struct(\"{}\")".format(title(self.name)))
        if self.super:
            super = self.structs[self.super]
            print("            .field(\"{n}\", &self.{n})".format(n=snake(super.name)))
        with indent(3):
            for m in self.values:
                m.emit_debug()
        print("            .finish()")
        print("    }")
        print("}")
    def emit_maybe_default(self):
        if self.name != ("sv", "tune", "params"):
            return
        if len(self.values) == 33:
            print("""\
pub const SV_TUNE_PARAMS_DEFAULT: SvTuneParams = SvTuneParams {
    ground_control_speed: TuneParam(1000),
    ground_control_accel: TuneParam(200),
    ground_friction: TuneParam(50),
    ground_jump_impulse: TuneParam(1320),
    air_jump_impulse: TuneParam(1200),
    air_control_speed: TuneParam(500),
    air_control_accel: TuneParam(150),
    air_friction: TuneParam(95),
    hook_length: TuneParam(38000),
    hook_fire_speed: TuneParam(8000),
    hook_drag_accel: TuneParam(300),
    hook_drag_speed: TuneParam(1500),
    gravity: TuneParam(50),
    velramp_start: TuneParam(55000),
    velramp_range: TuneParam(200000),
    velramp_curvature: TuneParam(140),
    gun_curvature: TuneParam(125),
    gun_speed: TuneParam(220000),
    gun_lifetime: TuneParam(200),
    shotgun_curvature: TuneParam(125),
    shotgun_speed: TuneParam(275000),
    shotgun_speeddiff: TuneParam(80),
    shotgun_lifetime: TuneParam(20),
    grenade_curvature: TuneParam(700),
    grenade_speed: TuneParam(100000),
    grenade_lifetime: TuneParam(200),
    laser_reach: TuneParam(80000),
    laser_bounce_delay: TuneParam(15000),
    laser_bounce_num: TuneParam(100),
    laser_bounce_cost: TuneParam(0),
    laser_damage: TuneParam(500),
    player_collision: TuneParam(100),
    player_hooking: TuneParam(100),
};
""")

class NetObject(Struct):
    const_type = "u16"
    def __init__(self, name, values, ex=None, validate_size=True):
        super().__init__(name, values, ex)
        if not validate_size:
            self.attributes.add("dont_validate_size")
    def emit_definition(self):
        print("#[repr(C)]")
        super().emit_definition()
    def emit_impl_encode_decode_int(self):
        import_(
            "common::slice",
            "error::Error",
            "packer::ExcessData",
            "packer::IntUnpacker",
            "warn::Warn",
        )
        if self.super:
            super = self.structs[self.super]
        else:
            super = None
        print("impl{l} {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
        print("        let result = Self::decode_inner(p)?;")
        print("        p.finish(warn);")
        print("        Ok(result)")
        print("    }")
        print("    pub fn decode_inner(_p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
        if self.values or super:
            print("        Ok({} {{".format(title(self.name)))
            if super:
                print("            {}: {}::decode_inner(_p)?,".format(snake(super.name), title(super.name), super.lifetime()))
            with indent(3):
                for m in self.values:
                    m.emit_decode_int()
            print("        })")
        else:
            print("        Ok({})".format(title(self.name)))
        print("    }")
        print("    pub fn encode(&self) -> &[i32] {")
        if super:
            print("        self.{}.encode();".format(snake(super.name)))
        with indent(2):
            for m in self.values:
                m.emit_assert()
        print("        unsafe { slice::transmute(slice::ref_slice(self)) }")
        print("    }")
        print("}")
    def int_size(self):
        size = sum(m.int_size() for m in self.values)
        if self.super:
            size += self.structs[self.super].int_size()
        return size
def NetObjectEx(name, ex, values, **kwargs):
    return NetObject(name, values, ex=ex, **kwargs)

class NetEvent(NetObject):
    pass
def NetEventEx(name, ex, values, **kwargs):
    return NetEvent(name, values, ex=ex, **kwargs)

class NetMessage(Struct):
    const_type = "i32"
def NetMessageEx(name, ex, values, **kwargs):
    return NetMessage(name, values, ex=ex, **kwargs)

class NetConnless(Struct):
    def __init__(self, name, id, values):
        super().__init__(name, values)
        self.id = id
    def emit_consts(self):
        print(r"""pub const {}: &'static [u8; 8] = b"\xff\xff\xff\xff{}";""".format(caps(self.name), self.id))
    def serialize(self):
        return {
            "id": [255] * 4 + list(self.id.encode()),
            "name": self.name,
            "members": [v.serialize() for v in self.values],
        }
    @staticmethod
    def deserialize(json_obj):
        name = snake(json_obj["name"])
        id = bytes(json_obj["id"])
        if id[:4] != b"\xff\xff\xff\xff":
            raise ProtocolSpecError("non-ffffffff header for connless ID not supported")
        id = id[4:].decode()
        return NetConnless(name, id, [deserialize_member(m) for m in json_obj["members"]])


class Member:
    def __init__(self, name, default=None):
        self.name = canonicalize(name)
        self.default = default
    def definition(self):
        return "{}: {}".format(snake(self.name), self.type_)
    def contains_lifetime(self):
        return "'a" in self.type_
    def update(self, parent, consts, enums, structs):
        return self
    def emit_decode(self):
        print("{}: {},".format(snake(self.name), self.decode_expr()))
    def emit_decode_int(self):
        print("{}: {},".format(snake(self.name), self.decode_int_expr()))
    def emit_assert(self):
        assertion = self.assert_expr("self.{}".format(snake(self.name)))
        if assertion is not None:
            print("{};".format(assertion))
    def emit_encode(self):
        print("{}?;".format(self.encode_expr("self.{}".format(snake(self.name)))))
    def emit_debug(self):
        print(".field(\"{}\", &{})".format(snake(self.name), self.debug_expr("self.{}".format(snake(self.name)))))
    def validate_expr(self, self_expr):
        pass
    def assert_expr(self, self_expr):
        pass
    def debug_expr(self, self_expr):
        return self_expr
    def serialize(self):
        result = {}
        result["name"] = self.name
        if self.default is not None:
            result["default"] = self.default
        result["type"] = self.serialize_type()
        return result

class NetArray(Member):
    kind = "array"
    def __init__(self, *args):
        if len(args) == 2:
            inner, count = args
            name = inner.name
        else:
            name, inner, count = args
        super().__init__(name)
        self.inner = inner
        self.count = count
        self.type_ = "[{}; {}]".format(inner.type_, count)
    def decode_expr(self):
        return "[\n{}]".format("".join(
            "    {},\n".format(self.inner.decode_expr()) for _ in range(self.count)
        ))
    def emit_assert(self):
        assert_expr = self.inner.assert_expr("e")
        if assert_expr:
            print("for &e in &self.{} {{".format(snake(self.name)))
            print("    {};".format(assert_expr))
            print("}")
    def emit_encode(self):
        print("for &e in &self.{} {{".format(snake(self.name)))
        print("    {}?;".format(self.inner.encode_expr("e")))
        print("}")
    def decode_int_expr(self):
        return "[\n{}]".format("".join(
            "    {},\n".format(self.inner.decode_int_expr()) for _ in range(self.count)
        ))
    def debug_expr(self, self_expr):
        if self.inner.debug_expr("x") == "x":
            return self_expr
        import_("gamenet_common::debug::DebugSlice")
        return "DebugSlice::new(&{}, |e| {})".format(self_expr, self.inner.debug_expr("e"))
    def int_size(self):
        return self.inner.int_size() * self.count
    def serialize_type(self):
        return {
            "kind": self.kind,
            "count": self.count,
            "member_type": self.inner.serialize_type(),
        }
    @staticmethod
    def deserialize(name, json_obj):
        return NetArray(
            name,
            deserialize_member(json_obj["member_type"]),
            json_obj["count"],
        )

class NetOptional(Member):
    kind = "optional"
    def __init__(self, name, inner):
        super().__init__(name)
        self.inner = inner
        self.type_ = "Option<{}>".format(inner.type_)
    def decode_expr(self):
        END="?"
        inner_decode = self.inner.decode_expr()
        if not inner_decode.endswith(END):
            raise ValueError("can't form an optional of this type")
        return "{}.ok()".format(inner_decode[:-len(END)])
    def encode_expr(self, self_expr):
        return self.inner.encode_expr("{}.unwrap()").format(self_expr)
    def debug_expr(self, self_expr):
        return "{}.as_ref().map(|v| {})".format(self_expr, self.inner.debug_expr("v"))
    def assert_expr(self, self_expr):
        return "assert!({}.is_some())".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind, "inner": self.inner.serialize_type()}
    @staticmethod
    def deserialize(name, json_obj):
        return NetOptional(name, deserialize_member(json_obj["inner"]))

class NetString(Member):
    kind = "string"
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "_p.read_string()?"
    def encode_expr(self, self_expr):
        return "_p.write_string({})".format(self_expr)
    def debug_expr(self, self_expr):
        import_("common::pretty")
        return "pretty::Bytes::new(&{})".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind, "disallow_cc": False}
    @staticmethod
    def deserialize(name, json_obj):
        if json_obj["disallow_cc"]:
            return NetStringStrict(name)
        else:
            return NetString(name)

class NetStringStrict(NetString):
    def decode_expr(self):
        import_("packer::sanitize")
        return "sanitize(warn, {})?".format(super().decode_expr())
    def assert_expr(self, self_expr):
        import_(
            "packer::sanitize",
            "warn::Panic",
        )
        return "sanitize(&mut Panic, {}).unwrap()".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind, "disallow_cc": True}
NetStringHalfStrict = NetStringStrict

class NetData(Member):
    kind = "data"
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "_p.read_data(warn)?"
    def encode_expr(self, self_expr):
        return "_p.write_data({})".format(self_expr)
    def debug_expr(self, self_expr):
        import_("common::pretty")
        return "pretty::Bytes::new(&{})".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind, "size": "specified_before"}
    @staticmethod
    def deserialize(name, json_obj):
        return NetData(name)

class NetDataRest(Member):
    kind = "rest"
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "_p.read_rest()?"
    def encode_expr(self, self_expr):
        return "_p.write_rest({})".format(self_expr)
    def debug_expr(self, self_expr):
        import_("common::pretty")
        return "pretty::Bytes::new(&{})".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetDataRest(name)

class NetSha256(Member):
    type_ = "Sha256"
    kind = "sha256"
    def decode_expr(self):
        import_("common::digest::Sha256")
        return "Sha256::from_slice(_p.read_raw(32)?).unwrap()"
    def encode_expr(self, self_expr):
        return "_p.write_raw(&{}.0)".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetSha256(name)

class NetUuid(Member):
    type_ = "Uuid"
    kind = "uuid"
    def decode_expr(self):
        import_("uuid::Uuid")
        return "Uuid::from_slice(_p.read_raw(16)?).unwrap()"
    def encode_expr(self, self_expr):
        return "_p.write_raw({}.as_bytes())".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetUuid(name)

class NetIntAny(Member):
    kind = "int32"
    type_ = "i32"
    def decode_expr(self):
        return "_p.read_int(warn)?"
    def encode_expr(self, self_expr):
        return "_p.write_int({})".format(self_expr)
    def decode_int_expr(self):
        return "_p.read_int()?"
    def int_size(self):
        return 1
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        if "min" in json_obj and "max" not in json_obj:
            if json_obj["min"] == 0:
                return NetIntPositive(name)
            else:
                return NetIntMin(name, json_obj["min"])
        elif "min" in json_obj or "max" in json_obj:
            return NetIntRange(name, json_obj["min"], json_obj["max"])
        else:
            return NetIntAny(name)

def import_consts(value):
    value = str(value)
    for const in "FLAG_MISSING MAX_CLIENTS SPEC_FREEVIEW TEAM_BLUE TEAM_RED".split():
        if const in value:
            import_("enums::{}".format(const))

def evaluate_constant(consts, enums, constant):
    try:
        return int(constant)
    except ValueError:
        pass
    offset = 0
    if constant.endswith("-1"):
        constant = constant[:-2]
        offset = -1
    if constant.startswith("NUM_") and constant.endswith("S"):
        return len(enums[canonicalize(constant[4:-1])].values) + offset
    c = canonicalize(constant)
    if c in consts:
        return consts[c].value + offset
    for i in range(1, len(c))[::-1]:
        if c[:i] in enums:
            try:
                index = enums[c[:i]].values.index(c[i:])
            except ValueError:
                pass
            else:
                return index + enums[c[:i]].offset
    raise ProtocolSpecError("unevaluatable constant {}".format(constant))

class NetIntRange(NetIntAny):
    def __init__(self, name, min, max):
        super().__init__(name)
        self.min = min
        self.max = max
    def update(self, parent, consts, enums, structs):
        min = str(self.min)
        max = str(self.max)
        if max == "max_int":
            if min == "0":
                return NetIntPositive(self.name)
            elif min == "min_int":
                return NetIntAny(self.name)
            else:
                return NetIntMin(self.name, self.min)
        if min == "TEAM_SPECTATORS" and max == "TEAM_BLUE":
            return NetEnum(self.name, "team")
        if parent.name == ("player", "input") and self.name == ("player", "flags") and min == "0" and max == "256":
            return NetIntAny(self.name)
        elif parent.name == ("player", "input") and self.name == ("wanted", "weapon") and min == "0" and max == "NUM_WEAPONS-1":
            max = "NUM_WEAPONS"
        if self.name == ("hooked", "player") and min == "0":
            min = "-1"
        elif self.name == ("emote",) and max == str(len(enums[("emote",)].values)):
            max = "NUM_EMOTES-1"
        elif max == "NUM_SPECMODES-1":
            max = "NUM_SPECS-1"
        if min == "0" and max.startswith("NUM_") and max.endswith("S-1"):
            return NetEnum(self.name, max[4:-3])
        if max == "NUM_WEAPONS-1":
            max = len(enums[("weapon",)].values) - 1
        self.min = evaluate_constant(consts, enums, min)
        self.max = evaluate_constant(consts, enums, max)
        return self
    def decode_expr(self):
        import_("packer::in_range")
        import_consts(self.min)
        import_consts(self.max)
        return "in_range({}, {}, {})?".format(super().decode_expr(), self.min, self.max)
    def assert_expr(self, self_expr):
        import_consts(self.min)
        import_consts(self.max)
        return "assert!({} <= {s} && {s} <= {})".format(self.min, self.max, s=self_expr)
    def decode_int_expr(self):
        import_("packer::in_range")
        import_consts(self.min)
        import_consts(self.max)
        return "in_range({}, {}, {})?".format(super().decode_int_expr(), self.min, self.max)
    def serialize_type(self):
        return {"kind": self.kind, "min": self.min, "max": self.max}

class NetIntPositive(NetIntAny):
    def __init__(self, name):
        super().__init__(name)
    def decode_expr(self):
        import_("packer::positive")
        return "positive({})?".format(super().decode_expr())
    def assert_expr(self, self_expr):
        return "assert!({} >= 0)".format(self_expr)
    def decode_int_expr(self):
        import_("packer::positive")
        return "positive({})?".format(super().decode_int_expr())
    def serialize_type(self):
        return {"kind": self.kind, "min": 0}

class NetIntMin(NetIntAny):
    def __init__(self, name, min):
        super().__init__(name)
        self.min = min
    def decode_expr(self):
        import_("packer::at_least")
        return "at_least({}, {})?".format(super().decode_expr(), self.min)
    def assert_expr(self, self_expr):
        return "assert!({} >= {})".format(self_expr, self.min)
    def decode_int_expr(self):
        import_("packer::at_least")
        return "at_least({}, {})?".format(super().decode_int_expr(), self.min)
    def serialize_type(self):
        return {"kind": self.kind, "min": self.min}

class NetEnum(NetIntAny):
    kind = "enum"
    def __init__(self, name, enum_name):
        super().__init__(name)
        if isinstance(enum_name, Enum):
            enum_name = enum_name.name
        self.enum_name = canonicalize(enum_name)
        self.type_ = "enums::{}".format(title(self.enum_name))
    def decode_expr(self):
        import_("enums")
        return "enums::{}::from_i32({})?".format(title(self.enum_name), super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.to_i32()".format(self_expr))
    def decode_int_expr(self):
        import_("enums")
        return "enums::{}::from_i32({})?".format(title(self.enum_name), super().decode_int_expr())
    def serialize_type(self):
        return {"kind": self.kind, "enum": self.enum_name}
    @staticmethod
    def deserialize(name, json_obj):
        return NetEnum(name, tuple(json_obj["enum"]))

class NetFlag(NetIntAny):
    kind = "flags"
    def __init__(self, name, flags_name):
        super().__init__(name)
        if isinstance(flags_name, Flags):
            flags_name = flags_name.name
        self.flags_name = canonicalize(flags_name)
    def serialize_type(self):
        return {"kind": self.kind, "flags": self.flags_name}
    @staticmethod
    def deserialize(name, json_obj):
        return NetFlag(name, tuple(json_obj["flags"]))

class NetBool(NetIntAny):
    kind = "boolean"
    type_ = "bool"
    def decode_expr(self):
        import_("packer::to_bool")
        return "to_bool({})?".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{} as i32".format(self_expr))
    def decode_int_expr(self):
        import_("packer::to_bool")
        return "to_bool({})?".format(super().decode_int_expr())
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj, default=None):
        return NetBool(name, default=default)

class NetTuneParam(NetIntAny):
    kind = "tune_param"
    type_ = "TuneParam"
    def decode_expr(self):
        return "TuneParam({})".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.0".format(self_expr))
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetTuneParam(name)

class NetTick(NetIntAny):
    kind = "tick"
    type_ = "::snap_obj::Tick"
    def decode_expr(self):
        return "::snap_obj::Tick({})".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.0".format(self_expr))
    def decode_int_expr(self):
        return "::snap_obj::Tick({})".format(super().decode_int_expr())
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetTick(name)

class NetObjectMember(Member):
    kind = "snapshot_object"
    def __init__(self, name, type_):
        super().__init__(name)
        self.type_ = "::snap_obj::{}".format(title(type_))
        self.type_name = type_
    def decode_expr(self):
        return "{}::decode_msg(warn, _p)?".format(self.type_)
    def encode_expr(self, self_expr):
        import_("packer::with_packer")
        return "with_packer(&mut _p, |p| {}.encode_msg(p))".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind, "name": self.type_name}
    @staticmethod
    def deserialize(name, json_obj):
        return NetObjectMember(name, tuple(json_obj["name"]))

class NetAddrs(Member):
    kind = "packed_addresses"
    type_ = "&'a [AddrPacked]"
    def definition(self):
        import_("super::AddrPacked")
        return super().definition()
    def decode_expr(self):
        import_(
            "gamenet_common::msg::AddrPackedSliceExt",
            "warn::wrap",
        )
        return "AddrPackedSliceExt::from_bytes(wrap(warn), _p.read_rest()?)"
    def encode_expr(self, self_expr):
        return "_p.write_rest({}.as_bytes())".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetAddrs(name)

class NetBigEndianU16(Member):
    kind = "be_uint16"
    type_ = "u16"
    def decode_expr(self):
        import_("common::num::BeU16")
        return "{ let s = _p.read_raw(2)?; BeU16::from_bytes(&[s[0], s[1]]).to_u16() }"
    def encode_expr(self, self_expr):
        import_("common::num::BeU16")
        return "_p.write_raw(BeU16::from_u16({}).as_bytes())".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetBigEndianU16(name)

class NetU8(Member):
    kind = "uint8"
    type_ = "u8"
    def decode_expr(self):
        return "_p.read_raw(1)?[0]"
    def encode_expr(self, self_expr):
        return "_p.write_raw(&[{}])".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetU8(name)

class NetIntString(NetString):
    kind = "int32_string"
    type_ = "i32"
    def decode_expr(self):
        import_("gamenet_common::msg::int_from_string")
        return "int_from_string(_p.read_string()?)?"
    def encode_expr(self, self_expr):
        import_("gamenet_common::msg::string_from_int")
        return "_p.write_string(&string_from_int({}))".format(self_expr)
    def debug_expr(self, self_expr):
        return self_expr
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetIntString(name)

class NetClients(Member):
    kind = "serverinfo_client"
    type_ = "ClientsData<'a>"
    def definition(self):
        import_("super::ClientsData")
        return super().definition()
    def decode_expr(self):
        import_("super::ClientsData")
        return "ClientsData::from_bytes(_p.read_rest()?)"
    def encode_expr(self, self_expr):
        return "_p.write_rest({}.as_bytes())".format(self_expr)
    def serialize_type(self):
        return {"kind": self.kind}
    @staticmethod
    def deserialize(name, json_obj):
        return NetClients(name)

class Constant:
    def __init__(self, name, value):
        if isinstance(value, int):
            type = "int32"
        elif isinstance(value, str):
            type = "string"
        else:
            raise TypeError("value must be an int or a string")
        self.name = canonicalize(name)
        self.type = type
        self.value = value
    def emit_definition(self):
        if self.type == "int32":
            value = str(self.value)
        else:
            value = '"{}"'.format(self.value.replace('"', '\\"'))
        type = "i32" if self.type == "int32" else "&'static str"
        print("pub const {}: {} = {};".format(caps(self.name), type, value))
    def serialize(self):
        return {"name": self.name, "type": self.type, "value": self.value}
    @staticmethod
    def deserialize(json_obj):
        type = json_obj["type"]
        value = json_obj["value"]
        if type == "string":
            if not isinstance(value, str):
                raise ProtocolSpecError("invalid string value {!r}".format(value))
        elif type == "int32":
            if not isinstance(value, int):
                raise ProtocolSpecError("invalid int32 value {!r}".format(value))
        else:
            raise ProtocolSpecError("unknown constant type {!r}".format(type))
        return Constant(tuple(json_obj["name"]), json_obj["value"])

MEMBER_TYPES = [
    NetArray,
    NetOptional,
    NetString,
    NetData,
    NetDataRest,
    NetSha256,
    NetUuid,
    NetIntAny,
    NetEnum,
    NetFlag,
    NetBool,
    NetTuneParam,
    NetTick,
    NetObjectMember,
    NetAddrs,
    NetBigEndianU16,
    NetU8,
    NetIntString,
    NetClients,
]
MEMBER_TYPES_MAPPING = {t.kind: t for t in MEMBER_TYPES}
if len(MEMBER_TYPES) != len(MEMBER_TYPES_MAPPING):
    raise RuntimeError("duplicate member type kind")
