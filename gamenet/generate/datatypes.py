from collections import namedtuple

def title(c):
    return "".join(p.title() for p in c)

def snake(c):
    return "_".join(c)

def caps(c):
    return "_".join(p.upper() for p in c)

def canonicalize(s):
    result = canonicalize_impl(s)
    if result == ("type",):
        result = ("type_",)
    return result

def canonicalize_impl(s):
    if isinstance(s, tuple):
        return s
    if s.isupper():
        return tuple(p.lower() for p in s.split("_"))
    PREFIXES=["m_p", "m_a", "m_"]
    for prefix in PREFIXES:
        if s.startswith(prefix):
            s = s[len(prefix):]
    s = s.replace("ID", "Id")
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

class NameValues:
    def __init__(self, name, values):
        names = name.split(':')
        if not 1 <= len(names) <= 2:
            raise ValueError("invalid name format")
        self.name = canonicalize(names[0])
        self.super = canonicalize(names[1]) if len(names) == 2 else None
        self.values = values
    def init(self, index, enums, structs):
        self.index = index
        self.enums = enums
        self.structs = structs

def emit_header_enums():
    print("""\
use packer::IntOutOfRange;

pub const MAX_CLIENTS: i32 = 16;
pub const SPEC_FREEVIEW: i32 = -1;

pub const FLAG_MISSING: i32 = -3;
pub const FLAG_ATSTAND: i32 = -2;
pub const FLAG_TAKEN: i32 = -1;
""")

def emit_header_snap_obj():
    print("""\
use debug::DebugSlice;
use enums::*;
use error::Error;
use packer::ExcessData;
use packer::IntUnpacker;
use packer::in_range;
use packer::positive;
use std::fmt;
use warn::Warn;

#[derive(Clone, Copy, Debug)]
pub struct Tick(i32);
""")

def emit_header_msg_game():
    print("""\
use buffer::CapacityError;
use common::pretty;
use debug::DebugSlice;
use enums::*;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::in_range;
use packer::sanitize;
use packer::to_bool;
use packer::with_packer;
use std::fmt;
use super::SystemOrGame;
use warn::Panic;
use warn::Warn;

impl<'a> Game<'a> {
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::Game(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode_msg(p)));
        Ok(p.written())
    }
}

pub const CL_CALL_VOTE_TYPE_OPTION: &'static [u8] = b"option";
pub const CL_CALL_VOTE_TYPE_KICK: &'static [u8] = b"kick";
pub const CL_CALL_VOTE_TYPE_SPEC: &'static [u8] = b"spectate";
""")

def emit_enum(name, structs):
    name = canonicalize(name)
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    print("#[derive(Clone, Copy)]")
    print("pub enum {}{} {{".format(title(name), lifetime))
    for s in structs:
        print("    {}({}{}),".format(title(s.name), title(s.name), s.lifetime()))
    print("}")
    print()
    print("impl{l} {}{l} {{".format(title(name), l=lifetime))
    print("    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: i32, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(title(name), l=lifetime))
    print("        Ok(match msg_id {")
    for s in structs:
        print("            {} => {}::{s}(try!({s}::decode(warn, _p))),".format(caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownMessage),".format(caps(s.name), title(name), s=title(s.name)))
    print("        })")
    print("    }")
    print("    pub fn msg_id(&self) -> i32 {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(_) => {},".format(title(name), title(s.name), caps(s.name)))
    print("        }")
    print("    }")
    print("    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.encode(p),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")
    print("")
    print("impl{l} fmt::Debug for {}{l} {{".format(title(name), l=lifetime))
    print("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(ref i) => i.fmt(f),".format(title(name), title(s.name)))
    print("        }")
    print("    }")
    print("}")

class Enum(NameValues):
    def __init__(self, name, values, offset=0):
        super().__init__(name, [canonicalize(v) for v in values])
        self.offset = offset
    def emit_definition(self):
        for i, name in enumerate(self.values):
            print("pub const {}_{}: i32 = {};".format(caps(self.name), caps(name), i + self.offset))
        print()
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

class Flags(NameValues):
    def __init__(self, name, values):
        super().__init__(name, [canonicalize(v) for v in values])
    def emit_definition(self):
        for i, name in enumerate(self.values):
            print("pub const {}_{}: i32 = 1 << {};".format(caps(self.name), caps(name), i))

def lifetime(members):
    return "<'a>" if any(m.contains_lifetime() for m in members) else ""

class Struct(NameValues):
    def lifetime(self):
        result = lifetime(self.values)
        if self.super:
            result = result or self.structs[self.super].lifetime()
        return result
    def init(self, index, enums, structs):
        super().init(index, enums, structs)

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
                if m is not None:
                    self.values.append(m)

        self.values = [member.update(enums, structs) for member in self.values]

    def emit_consts(self):
        print("pub const {}: i32 = {};".format(caps(self.name), self.index))
    def emit_definition(self):
        if self.super:
            super = self.structs[self.super]
        else:
            super = None

        print("#[derive(Clone, Copy)]")
        if self.values or super:
            print("pub struct {}{} {{".format(title(self.name), self.lifetime()))
            if super:
                print("    pub {}: {}{},".format(snake(super.name), title(super.name), super.lifetime()))
            for member in self.values:
                print("    pub {},".format(member.definition()))
            print("}")
        else:
            print("pub struct {};".format(title(self.name)))
    def emit_impl_encode_decode(self):
        print("impl{l} {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
        if self.values:
            print("        let result = Ok({} {{".format(title(self.name)))
            for m in self.values:
                m.emit_decode()
            print("        });")
        else:
            print("        let result = Ok({});".format(title(self.name)))
        print("        _p.finish(warn);")
        print("        result")
        print("    }")
        print("    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {{".format(title(self.name), l=self.lifetime()))
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
        for m in self.values:
            m.emit_debug()
        print("            .finish()")
        print("    }")
        print("}")

class NetObject(Struct):
    def emit_impl_encode_decode_int(self):
        if self.super:
            super = self.structs[self.super]
        else:
            super = None

        print("impl{l} {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
        print("        let result = try!(Self::decode_inner(p));")
        print("        p.finish(warn);")
        print("        Ok(result)")
        print("    }")
        print("    pub fn decode_inner(_p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
        if self.values or super:
            print("        Ok({} {{".format(title(self.name)))
            if super:
                print("            {}: try!({}::decode_inner(_p)),".format(snake(super.name), title(super.name), super.lifetime()))
            for m in self.values:
                m.emit_decode_int()
            print("        })")
        else:
            print("        Ok({})".format(title(self.name)))
        print("    }")
        print("}")

class NetEvent(NetObject): pass
class NetMessage(Struct): pass

class Member:
    def __init__(self, name):
        self.name = canonicalize(name)
    def definition(self):
        return "{}: {}".format(snake(self.name), self.type_)
    def contains_lifetime(self):
        return "&'a" in self.type_
    def update(self, enums, structs):
        return self
    def emit_decode(self):
        print("            {}: {},".format(snake(self.name), self.decode_expr()))
    def emit_decode_int(self):
        print("            {}: {},".format(snake(self.name), self.decode_int_expr()))
    def emit_assert(self):
        assertion = self.assert_expr("self.{}".format(snake(self.name)))
        if assertion is not None:
            print("        {};".format(assertion))
    def emit_encode(self):
        print("        try!({});".format(self.encode_expr("self.{}".format(snake(self.name)))))
    def emit_debug(self):
        print("            .field(\"{}\", &{})".format(snake(self.name), self.debug_expr("self.{}".format(snake(self.name)))))
    def validate_expr(self, self_expr):
        pass
    def assert_expr(self, self_expr):
        pass
    def debug_expr(self, self_expr):
        return self_expr

class NetArray(Member):
    def __init__(self, name, inner, count):
        super().__init__(name)
        self.inner = inner
        self.count = count
        self.type_ = "[{}; {}]".format(inner.type_, count)
    def decode_expr(self):
        indent = "\n" + 4*4*" "
        return "[{}{},\n            ]".format(
                indent,
                (","+indent).join(self.inner.decode_expr() for _ in range(self.count))
        )
    def emit_assert(self):
        print("        for e in &self.{} {{".format(snake(self.name)))
        print("            {};".format(self.inner.assert_expr("e")))
        print("        }")
    def emit_encode(self):
        print("        for e in &self.{} {{".format(snake(self.name)))
        print("            try!({});".format(self.inner.encode_expr("e")))
        print("        }")
    def decode_int_expr(self):
        indent = "\n" + 4*4*" "
        return "[{}{},\n            ]".format(
                indent,
                (","+indent).join(self.inner.decode_int_expr() for _ in range(self.count))
        )
    def debug_expr(self, self_expr):
        return "DebugSlice::new(&{}, |e| {})".format(self_expr, self.inner.debug_expr("e"))

class NetString(Member):
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "try!(_p.read_string())"
    def encode_expr(self, self_expr):
        return "_p.write_string({})".format(self_expr)
    def debug_expr(self, self_expr):
        return "pretty::Bytes::new(&{})".format(self_expr)

class NetStringStrict(NetString):
    def decode_expr(self):
        return "try!(sanitize(warn, {}))".format(super().decode_expr())
    def assert_expr(self, self_expr):
        return "sanitize(&mut Panic, {}).unwrap()".format(self_expr)

class NetIntAny(Member):
    type_ = "i32"
    def decode_expr(self):
        return "try!(_p.read_int(warn))"
    def encode_expr(self, self_expr):
        return "_p.write_int({})".format(self_expr)
    def decode_int_expr(self):
        return "try!(_p.read_int())"

class NetIntRange(NetIntAny):
    def __init__(self, name, min, max):
        super().__init__(name)
        self.min = min
        self.max = max
    def update(self, enums, structs):
        min = str(self.min)
        max = str(self.max)
        if min == "0" and max == "max_int":
            return NetIntPositive(self.name)
        if self.name == "emote" and max == str(len(enums[("emote",)]) - 1):
            max = "NUM_EMOTES-1"
        if str(self.min) == "0" and max.startswith("NUM_") and max.endswith("S-1"):
            enum_name = canonicalize(self.max[4:-3])
            return NetEnum(self.name, enum_name)
        if max == "NUM_WEAPONS-1":
            self.max = len(enums[("weapon",)].values) - 1
        return self
    def decode_expr(self):
        return "try!(in_range({}, {}, {}))".format(super().decode_expr(), self.min, self.max)
    def assert_expr(self, self_expr):
        return "assert!({} <= {s} && {s} <= {})".format(self.min, self.max, s=self_expr)
    def decode_int_expr(self):
        return "try!(in_range({}, {}, {}))".format(super().decode_int_expr(), self.min, self.max)

class NetIntPositive(NetIntAny):
    def __init__(self, name):
        super().__init__(name)
    def update(self, enums, structs):
        return self
    def decode_expr(self):
        return "try!(positive({}))".format(super().decode_expr())
    def assert_expr(self, self_expr):
        return "assert!({} >= 0)".format(self_expr)
    def decode_int_expr(self):
        return "try!(positive({}))".format(super().decode_int_expr())

class NetEnum(NetIntAny):
    def __init__(self, name, enum_name):
        super().__init__(name)
        self.enum_name = enum_name
        self.type_ = title(enum_name)
    def decode_expr(self):
        return "try!({}::from_i32({}))".format(title(self.enum_name), super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.to_i32()".format(self_expr))
    def decode_int_expr(self):
        return "try!({}::from_i32({}))".format(title(self.enum_name), super().decode_int_expr())

class NetBool(NetIntAny):
    type_ = "bool"
    def decode_expr(self):
        return "try!(to_bool({}))".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{} as i32".format(self_expr))

class NetTick(NetIntAny):
    type_ = "Tick"
    def decode_int_expr(self):
        return "Tick({})".format(super().decode_int_expr())
