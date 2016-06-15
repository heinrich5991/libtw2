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

def emit_header():
    print("""\
use buffer::CapacityError;
use bytes::PrettyBytes;
use error::ControlCharacters;
use error::Error;
use error::IntOutOfRange;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::with_packer;
use std::fmt;
use super::SystemOrGame;
use warn::Panic;
use warn::Warn;

fn in_range(v: i32, min: i32, max: i32) -> Result<i32, IntOutOfRange> {
    if min <= v && v <= max {
        Ok(v)
    } else {
        Err(IntOutOfRange)
    }
}

fn to_bool(v: i32) -> Result<bool, IntOutOfRange> {
    Ok(try!(in_range(v, 0, 1)) != 0)
}

fn sanitize<'a, W: Warn<Warning>>(warn: &mut W, v: &'a [u8])
    -> Result<&'a [u8], ControlCharacters>
{
    if v.iter().any(|&b| b < b' ') {
        return Err(ControlCharacters);
    }
    let _ = warn;
    // TODO: Implement whitespace skipping.
    Ok(v)
}

impl<'a> Game<'a> {
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::Game(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode_msg(p)));
        Ok(p.written())
    }
}

pub const MAX_CLIENTS: i32 = 16;
pub const SPEC_FREEVIEW: i32 = -1;
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
        for name in self.values:
            print("    {},".format(title(name)))
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
        self.values = [member.update(enums, structs) for member in self.values]
    def emit_consts(self):
        print("pub const {}: i32 = {};".format(caps(self.name), self.index))
    def emit_definition(self):
        if self.super:
            super = structs[self.super]
        else:
            super = None

        print("#[derive(Clone, Copy)]")
        if self.values or super:
            lifetime = self.lifetime()
            print("pub struct {}{} {{".format(title(self.name), self.lifetime()))
            if super:
                print("    pub {}: {}{},".format(snake(super.name), title(super.name), super.lifetime()))
            for member in self.values:
                print("    pub {},".format(member.definition()))
            print("}")
        else:
            print("pub struct {};".format(title(self.name)))
    def emit_impl(self):
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
        print("impl{l} fmt::Debug for {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
        print("        f.debug_struct(\"{}\")".format(title(self.name)))
        for m in self.values:
            m.emit_debug()
        print("            .finish()")
        print("    }")
        print("}")

class NetObject(Struct): pass
class NetEvent(Struct): pass
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
    def emit_assert(self):
        assertion = self.assert_expr("self.{}".format(snake(self.name)))
        if assertion is not None:
            print("        {};".format(assertion))
    def emit_encode(self):
        print("        try!({});".format(self.encode_expr("self.{}".format(snake(self.name)))))
    def emit_debug(self):
        print("            .field(\"{}\", &{})".format(snake(self.name), self.debug_expr("self.{}".format(snake(self.name)))))
    def assert_expr(self, self_expr):
        pass
    def debug_expr(self, self_expr):
        return self_expr

class NetString(Member):
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "try!(_p.read_string())"
    def encode_expr(self, self_expr):
        return "_p.write_string({})".format(self_expr)
    def debug_expr(self, self_expr):
        return "PrettyBytes::new(&{})".format(self_expr)

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

class NetIntRange(NetIntAny):
    def __init__(self, name, min, max):
        super().__init__(name)
        self.min = min
        self.max = max
    def update(self, enums, structs):
        max = str(self.max)
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

class NetEnum(NetIntAny):
    def __init__(self, name, enum_name):
        super().__init__(name)
        self.enum_name = enum_name
        self.type_ = title(enum_name)
    def decode_expr(self):
        return "try!({}::from_i32({}))".format(title(self.enum_name), super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.to_i32()".format(self_expr))

class NetBool(NetIntAny):
    type_ = "bool"
    def decode_expr(self):
        return "try!(to_bool({}))".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{} as i32".format(self_expr))

class NetTick(Member):
    type_ = "Tick"
