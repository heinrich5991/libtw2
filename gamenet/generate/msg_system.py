import datatypes

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
    (16, "input", "", ["i:ack_snapshot", "i:intended_tick", "i:input_size", "inp:input"]),
    (17, "rcon_cmd", "vital", ["s:cmd"]),
    (18, "rcon_auth", "vital", ["s:_unused", "s:password", "i?:request_commands"]),
    (19, "request_map_data", "vital", ["i:chunk"]),
    (20, "ping", "", []),
    (21, "ping_reply", "", []),
    (25, "rcon_cmd_add", "vital", ["s:name", "s:help", "s:params"]),
    (26, "rcon_cmd_remove", "vital", ["s:name"]),
]

header = """\
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
                new_type = datatypes.NetString
            elif type_ == 'i':
                new_type = datatypes.NetIntAny
            elif type_ == 'd':
                new_type = datatypes.NetData
            elif type_ == 'inp':
                new_type = lambda name: datatypes.NetStruct(name, "::snap_obj::PlayerInput")
            else:
                raise ValueError("Invalid member: {:?}".format(member))
            member = new_type(member_name)
            if optional:
                member = datatypes.NetOptional(member_name, member)
            result_members.append(member)
        result.append(datatypes.NetMessage(name, result_members))

    for (msg_id, _, _, _), struct in zip(msgs, result):
        struct.init(msg_id, [], result)

    return result

def main():
    msgs = make_msgs(NETMSGS_SYSTEM)
    emit = datatypes.Emit()
    with emit:
        generate_header()
        datatypes.emit_enum_msg_module("System", msgs)
    emit.dump()

if __name__ == '__main__':
    import sys
    sys.exit(main())
