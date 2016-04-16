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
    (16, "input", "", ["i:ack_snapshot", "i:intended_tick", "is:input"]),
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
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::with_packer;

#[derive(Clone, Copy)]
pub struct IntegerData<'a> {
    inner: &'a [u8],
}

impl<'a> IntegerData<'a> {
    fn from_bytes(bytes: &[u8]) -> IntegerData {
        IntegerData {
            inner: bytes,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        self.inner
    }
}

#[derive(Copy, Clone)]
enum SystemOrGame<S, G> {
    System(S),
    Game(G),
}

impl<S, G> SystemOrGame<S, G> {
    fn is_game(&self) -> bool {
        match *self {
            SystemOrGame::System(_) => false,
            SystemOrGame::Game(_) => true,
        }
    }
    fn is_system(&self) -> bool {
        !self.is_game()
    }
}

impl SystemOrGame<i32, i32> {
    fn decode_id(id: i32) -> SystemOrGame<i32, i32> {
        let sys = id & 1 != 0;
        let msg = id >> 1;
        if sys {
            SystemOrGame::System(msg)
        } else {
            SystemOrGame::Game(msg)
        }
    }
    fn internal_id(self) -> i32 {
        match self {
            SystemOrGame::System(msg) => msg,
            SystemOrGame::Game(msg) => msg,
        }
    }
    fn encode_id(self) -> i32 {
        let iid = self.internal_id() as u32;
        assert!((iid & (1 << 31)) == 0);
        let flag = if self.is_system() { 1 } else { 0 };
        ((iid << 1) | flag) as i32
    }
}

impl<'a> System<'a> {
    pub fn decode_complete(p: &mut Unpacker<'a>) -> Result<System<'a>, Error> {
        if let SystemOrGame::System(msg_id) = SystemOrGame::decode_id(try!(p.read_int())) {
            System::decode(msg_id, p)
        } else {
            Err(Error::new())
        }
    }
    pub fn encode_complete<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::System(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode(p)));
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
                new_type = 'integer_data'
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
    elif type_ == 'integer_data':
        result = "IntegerData<'a>"
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

def generate_mod_start(msgs):
    return """\
pub mod system {
    use buffer::CapacityError;
    use error::Error;
    use packer::Packer;
    use packer::Unpacker;
    use super::IntegerData;
"""

def generate_mod_end(msgs):
    return "}\n"

def generate_constants(msgs):
    result = []
    for msg_id, name, _ in msgs:
        result.append("    pub const {}: i32 = {};".format(const_name(name), msg_id))
    result.append("")
    return "\n".join(result)

def generate_structs(msgs):
    result = []
    for _, name, members in msgs:
        result.append("    #[derive(Clone, Copy)]")
        if members:
            result.append("    pub struct {}{} {{".format(struct_name(name), lifetime(members)))
            for type_, opt, name in members:
                result.append("        pub {}: {},".format(name, rust_type(type_, opt)))
            result.append("    }")
        else:
            result.append("    pub struct {};".format(struct_name(name)))
        result.append("")
    return "\n".join(result)

def generate_struct_impl(msgs):
    result = []
    for _, name, members in msgs:
        n = struct_name(name)
        l = lifetime(members)
        result.append("    impl{l} {}{l} {{".format(n, l=l))
        result.append("        pub fn decode(_p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(n, l=l))
        if members:
            result.append("            Ok({} {{".format(n))
            for type_, opt, name in members:
                if not opt:
                    conv = "try!({})".format
                else:
                    conv = "{}.ok()".format
                if type_ == 'string':
                    decode = "_p.read_string()"
                elif type_ == 'integer':
                    decode = "_p.read_int()"
                elif type_ == 'data':
                    decode = "_p.read_data()"
                elif type_ == 'integer_data':
                    decode = "_p.read_rest().map(IntegerData::from_bytes)"
                else:
                    raise ValueError("Invalid type: {}".format(type_))
                result.append(" "*4*4 + "{}: {},".format(name, conv(decode)))
            result.append("            })")
        else:
            result.append("            Ok({})".format(n))
        result.append("        }")
        if all(not opt for _, opt, _ in members):
            result.append("        pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)")
            result.append("            -> Result<&'d [u8], CapacityError>".format(n, l=l))
            result.append("        {")
            if members:
                for type_, _, name in members:
                    if type_ == 'string':
                        encode = "_p.write_string({})".format
                    elif type_ == 'integer':
                        encode = "_p.write_int({})".format
                    elif type_ == 'data':
                        encode = "_p.write_data({})".format
                    elif type_ == 'integer_data':
                        encode = "_p.write_rest({}.as_bytes())".format
                    else:
                        raise ValueError("Invalid type: {}".format(type_))
                    result.append(" "*4*3 + "try!({});".format(encode("self.{}".format(name))))
            result.append("            Ok(_p.written())")
            result.append("        }")
        result.append("    }")
        result.append("")
    return "\n".join(result)

def generate_system_extra(msgs):
    return system_extra

def generate_enum(msgs):
    result = []
    result.append("#[derive(Clone, Copy)]")
    result.append("pub enum System<'a> {")
    for _, name, members in msgs:
        result.append("    {s}(system::{s}{}),".format(lifetime(members), s=struct_name(name)))
    result.append("}")
    result.append("")
    return "\n".join(result)

def generate_enum_impl(msgs):
    result = []
    result.append("impl<'a> System<'a> {")
    result.append("    pub fn decode(msg_id: i32, p: &mut Unpacker<'a>) -> Result<System<'a>, Error> {")
    result.append("        use self::system::*;")
    result.append("        Ok(match msg_id {")
    for _, name, _ in msgs:
        result.append("            {} => System::{n}(try!({n}::decode(p))),".format(const_name(name), n=struct_name(name)))
    result.append("            _ => return Err(Error::new()),")
    result.append("        })")
    result.append("    }")
    result.append("    pub fn msg_id(&self) -> i32 {")
    result.append("        use self::system::*;")
    result.append("        match *self {")
    for _, name, members in msgs:
        result.append("            System::{}(_) => {},".format(struct_name(name), const_name(name)))
    result.append("        }")
    result.append("    }")
    result.append("    pub fn encode<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {")
    result.append("        match *self {")
    for _, name, _ in msgs:
        result.append("            System::{}(ref i) => i.encode(p),".format(struct_name(name), const_name(name)))
    result.append("        }")
    result.append("    }")
    result.append("}")
    result.append("")
    return "\n".join(result)

def main():
    msgs = make_msgs(NETMSGS_SYSTEM)
    steps = [
        generate_header,
        generate_mod_start,
        generate_constants,
        generate_structs,
        generate_struct_impl,
        generate_system_extra,
        generate_mod_end,
        generate_enum,
        generate_enum_impl,
    ]

    for g in steps:
        print(g(msgs))

if __name__ == '__main__':
    import sys
    sys.exit(main())
