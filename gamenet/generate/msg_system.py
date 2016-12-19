import re

NETMSGS_SYSTEM = [
    ( 1, "info", "vital", ["s:version", "s?:password"]),
    ( 2, "map_change", "vital", ["s:name", "i:crc", "i:size"]),
    ( 3, "map_data", "vital", ["i:last", "i:crc", "i:chunk", "d:data"]),
    ( 4, "con_ready", "vital", []),
    ( 5, "snap", "", ["i:tick", "i:delta_tick", "i:num_parts", "i:part", "i:crc", "d:data"]),
    ( 6, "snap_empty", "", ["i:tick", "i:delta_tick"]),
    ( 7, "snap_single", "", ["i:tick", "i:delta_tick", "i:crc", "d:data"]),
    ( 9, "input_timing", "", ["i:input_pred_tick", "i:time_left"]),
    (10, "rcon_auth_status", "vital", ["i?:auth_level", "i?:receive_commands"]),
    (11, "rcon_line", "vital", ["s:line"]),
    (14, "ready", "vital", []),
    (15, "enter_game", "vital", []),
    # TODO: Do checks on `input_size`
    (16, "input", "", ["i:ack_snapshot", "i:intended_tick", "i:input_size", "is:input"]),
    (17, "rcon_cmd", "vital", ["s:cmd"]),
    (18, "rcon_auth", "vital", ["s:_unused", "s:password", "i?:request_commands"]),
    (19, "request_map_data", "vital", ["i:chunk"]),
    (20, "ping", "", []),
    (21, "ping_reply", "", []),
    (25, "rcon_cmd_add", "vital", ["s:name", "s:help", "s:params"]),
    (26, "rcon_cmd_remove", "vital", ["s:name"]),
]

header = """\
use buffer::CapacityError;
use common::pretty;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::with_packer;
use snap_obj::PlayerInput;
use std::fmt;
use super::SystemOrGame;
use warn::Warn;

impl<'a> System<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<System<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::System(msg_id) =
            SystemOrGame::decode_id(try!(p.read_int(warn)))
        {
            System::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::System(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode_msg(p)));
        Ok(p.written())
    }
}
"""

system_extra = """\
impl<'a> Info<'a> {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.version));
        try!(_p.write_string(self.password.expect("Info needs a password")));
        Ok(_p.written())
    }
}
impl RconAuthStatus {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        assert!(self.auth_level.is_some() || self.receive_commands.is_none());
        try!(self.auth_level.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
        try!(self.receive_commands.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
        Ok(_p.written())
    }
}
impl<'a> RconAuth<'a> {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self._unused));
        try!(_p.write_string(self.password));
        try!(self.request_commands.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
        Ok(_p.written())
    }
}
"""

def make_msgs(msgs):
    result = []
    for msg_id, name, vital, members in msgs:
        result_members = []
        for member in members:
            type_, member_name = member.split(':')
            optional = False
            if type_.endswith("?"):
                optional = True
                type_ = type_[:-1]

            if type_ == 's':
                new_type = 'string'
            elif type_ == 'i':
                new_type = 'integer'
            elif type_ == 'd':
                new_type = 'data'
            elif type_ == 'is':
                new_type = 'input_data'
            else:
                raise ValueError("Invalid member: {:?}".format(member))
            result_members.append((new_type, optional, member_name))
        result.append((msg_id, name, result_members))

    return result

def struct_name(name):
    return name.title().replace('_', '')

def const_name(name):
    return name.upper()

def rust_type(type_, optional):
    if type_ == 'string':
        result = "&'a [u8]"
    elif type_ == 'integer':
        result = "i32"
    elif type_ == 'data':
        result = "&'a [u8]"
    elif type_ == 'input_data':
        result = "PlayerInput"
    else:
        raise ValueError("Invalid type: {}".format(type_))
    if not optional:
        return result
    else:
        return "Option<{}>".format(result)

def lifetime(members):
    for type_, _, _ in members:
        if "'a" in rust_type(type_, False):
            return "<'a>"
    return ""

def generate_header(msgs):
    return header

def generate_constants(msgs):
    result = []
    for msg_id, name, _ in msgs:
        result.append("pub const {}: i32 = {};".format(const_name(name), msg_id))
    result.append("")
    return "\n".join(result)

def generate_structs(msgs):
    result = []
    for _, name, members in msgs:
        result.append("#[derive(Clone, Copy)]")
        if members:
            result.append("pub struct {}{} {{".format(struct_name(name), lifetime(members)))
            for type_, opt, name in members:
                result.append("    pub {}: {},".format(name, rust_type(type_, opt)))
            result.append("}")
        else:
            result.append("pub struct {};".format(struct_name(name)))
        result.append("")
    return "\n".join(result)

def generate_struct_impl(msgs):
    result = []
    for _, name, members in msgs:
        n = struct_name(name)
        l = lifetime(members)
        result.append("impl{l} fmt::Debug for {}{l} {{".format(n, l=l))
        result.append("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
        result.append("        f.debug_struct(\"{}\")".format(n))
        for type_, opt, name in members:
            if not opt:
                conv = ".field(\"{name}\", &{conv}(self.{name})".format
            else:
                conv = "{}.ok()".format
            pretty = None
            if type_ == 'string' or type_ == 'data':
                pretty = "pretty::Bytes::new"
            if not pretty:
                conv = ".field(\"{name}\", &self.{name})"
            elif pretty and not opt:
                conv = ".field(\"{name}\", &{pretty}(&self.{name}))"
            elif pretty and opt:
                conv = ".field(\"{name}\", &self.{name}.as_ref().map(|{name}| {pretty}({name})))"
            else:
                raise RuntimeError("unreachable")
            result.append(" "*4*3 + conv.format(name=name, pretty=pretty))
        result.append("            .finish()")
        result.append("    }")
        result.append("}")
        result.append("impl{l} {}{l} {{".format(n, l=l))
        result.append("    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(n, l=l))
        if members:
            result.append("        let result = Ok({} {{".format(n))
            for type_, opt, name in members:
                if not opt:
                    conv = "try!({})".format
                else:
                    conv = "{}.ok()".format
                if type_ == 'string':
                    decode = "_p.read_string()"
                elif type_ == 'integer':
                    decode = "_p.read_int(warn)"
                elif type_ == 'data':
                    decode = "_p.read_data(warn)"
                elif type_ == 'input_data':
                    decode = "PlayerInput::decode_msg_inner(warn, _p)"
                else:
                    raise ValueError("Invalid type: {}".format(type_))
                result.append(" "*4*3 + "{}: {},".format(name, conv(decode)))
            result.append("        });")
        else:
            result.append("        let result = Ok({});".format(n))
        result.append("        _p.finish(warn);".format(n))
        result.append("        result".format(n))
        result.append("    }")
        if all(not opt for _, opt, _ in members):
            result.append("    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)")
            result.append("        -> Result<&'d [u8], CapacityError>".format(n, l=l))
            result.append("    {")
            if members:
                for type_, _, name in members:
                    if type_ == 'string':
                        encode = "_p.write_string({})".format
                    elif type_ == 'integer':
                        encode = "_p.write_int({})".format
                    elif type_ == 'data':
                        encode = "_p.write_data({})".format
                    elif type_ == 'input_data':
                        encode = "with_packer(&mut _p, |p| {}.encode_msg(p))".format
                    else:
                        raise ValueError("Invalid type: {}".format(type_))
                    result.append("        try!({});".format(encode("self.{}".format(name))))
            result.append("        Ok(_p.written())")
            result.append("    }")
        result.append("}")
        result.append("")
    return "\n".join(result)

def generate_system_extra(msgs):
    return system_extra

def generate_enum(msgs):
    result = []
    result.append("#[derive(Clone, Copy)]")
    result.append("pub enum System<'a> {")
    for _, name, members in msgs:
        result.append("    {s}({s}{}),".format(lifetime(members), s=struct_name(name)))
    result.append("}")
    result.append("")
    return "\n".join(result)

def generate_enum_impl(msgs):
    result = []
    result.append("impl<'a> System<'a> {")
    result.append("    pub fn decode_msg<W>(warn: &mut W, msg_id: i32, p: &mut Unpacker<'a>)")
    result.append("        -> Result<System<'a>, Error>")
    result.append("        where W: Warn<Warning>")
    result.append("    {")
    result.append("        Ok(match msg_id {")
    for _, name, _ in msgs:
        result.append("            {} => System::{n}(try!({n}::decode(warn, p))),".format(const_name(name), n=struct_name(name)))
    result.append("            _ => return Err(Error::UnknownId),")
    result.append("        })")
    result.append("    }")
    result.append("    pub fn msg_id(&self) -> i32 {")
    result.append("        match *self {")
    for _, name, members in msgs:
        result.append("            System::{}(_) => {},".format(struct_name(name), const_name(name)))
    result.append("        }")
    result.append("    }")
    result.append("    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {")
    result.append("        match *self {")
    for _, name, _ in msgs:
        result.append("            System::{}(ref i) => i.encode(p),".format(struct_name(name), const_name(name)))
    result.append("        }")
    result.append("    }")
    result.append("}")
    result.append("impl<'a> fmt::Debug for System<'a> {")
    result.append("    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {")
    result.append("        match *self {")
    for _, name, members in msgs:
        result.append("            System::{}(m) => m.fmt(f),".format(struct_name(name)))
    result.append("        }")
    result.append("    }")
    result.append("}")
    for _, name, members in msgs:
        l = lifetime(members)
        result.append("impl<'a> From<{}{}> for System<'a> {{".format(struct_name(name), l))
        result.append("    fn from(i: {}{}) -> System<'a> {{".format(struct_name(name), l))
        result.append("        System::{}(i)".format(struct_name(name)))
        result.append("    }")
        result.append("}")
    result.append("")
    return "\n".join(result)

def main():
    msgs = make_msgs(NETMSGS_SYSTEM)
    steps = [
        generate_header,
        generate_constants,
        generate_structs,
        generate_struct_impl,
        generate_system_extra,
        generate_enum,
        generate_enum_impl,
    ]

    for g in steps:
        print(g(msgs))

if __name__ == '__main__':
    import sys
    sys.exit(main())
