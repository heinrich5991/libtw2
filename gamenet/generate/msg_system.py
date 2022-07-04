import datatypes

SYSTEM_MSGS_0_5 = [
    ( 1, "info", "vital", "s:version s:name s:clan s:password"),
    ( 2, "map_change", "vital", "s:name i:crc"),
    ( 3, "map_data", "vital", "i:last i:total_size d:data"),
    ( 4, "snap", "", "i:tick i:delta_tick i:num_parts i:part i:crc d:data"),
    ( 5, "snap_empty", "", "i:tick i:delta_tick"),
    ( 6, "snap_single", "", "i:tick i:delta_tick i:crc d:data"),
    ( 8, "input_timing", "", "i:input_pred_tick i:time_left"),
    ( 9, "rcon_auth_status", "vital", "i:authed"),
    (10, "rcon_line", "vital", "s:line"),
    (13, "ready", "vital", ""),
    (14, "enter_game", "vital", ""),
    (15, "input", "", "i:ack_snapshot i:intended_tick i:input_size inp:input"),
    (16, "rcon_cmd", "vital", "s:cmd"),
    (17, "rcon_auth", "vital", "s:_unused s:password"),
    (18, "request_map_data", "vital", "i:chunk"),
    (21, "ping", "", ""),
    (22, "ping_reply", "", ""),
]

SYSTEM_MSGS_0_6 = [
    ( 1, "info", "vital", "s:version s?:password"),
    ( 2, "map_change", "vital", "s:name i:crc i:size"),
    ( 3, "map_data", "vital", "i:last i:crc i:chunk d:data"),
    ( 4, "con_ready", "vital", ""),
    ( 5, "snap", "", "i:tick i:delta_tick i:num_parts i:part i:crc d:data"),
    ( 6, "snap_empty", "", "i:tick i:delta_tick"),
    ( 7, "snap_single", "", "i:tick i:delta_tick i:crc d:data"),
    ( 9, "input_timing", "", "i:input_pred_tick i:time_left"),
    (10, "rcon_auth_status", "vital", "i?:auth_level i?:receive_commands"),
    (11, "rcon_line", "vital", "s:line"),
    (14, "ready", "vital", ""),
    (15, "enter_game", "vital", ""),
    # TODO: Do checks on `input_size`
    (16, "input", "", "i:ack_snapshot i:intended_tick i:input_size inp:input"),
    (17, "rcon_cmd", "vital", "s:cmd"),
    (18, "rcon_auth", "vital", "s:_unused s:password i?:request_commands"),
    (19, "request_map_data", "vital", "i:chunk"),
    (20, "ping", "", ""),
    (21, "ping_reply", "", ""),
    (25, "rcon_cmd_add", "vital", "s:name s:help s:params"),
    (26, "rcon_cmd_remove", "vital", "s:name"),
]

SYSTEM_MSGS_DDNET_15_2_5 = SYSTEM_MSGS_0_6 + [
    ("what-is@ddnet.tw", "what_is", "vital", "u:uuid"),
    ("it-is@ddnet.tw", "it_is", "vital", "u:uuid s:name"),
    ("i-dont-know@ddnet.tw", "i_dont_know", "vital", "u:uuid"),
    ("rcon-type@ddnet.tw", "rcon_type", "vital", "b:username_required"),
    ("map-details@ddnet.tw", "map_details", "vital", "s:name h:sha256 i:crc"),
    ("capabilities@ddnet.tw", "capabilities", "vital", "i:version i:flags"),
    ("clientver@ddnet.tw", "client_version", "vital", "u:connection_id i:ddnet_version s:ddnet_version_string"),
]

SYSTEM_MSGS_DDNET_16_2 = SYSTEM_MSGS_DDNET_15_2_5 + [
    ("ping@ddnet.tw", "ping_ex", "", "u:id"),
    ("pong@ddnet.tw", "pong_ex", "", "u:id"),
    ("checksum-request@ddnet.tw", "checksum_request", "vital", "u:id i:start i:length"),
    ("checksum-response@ddnet.tw", "checksum_response", "vital", "u:id h:sha256"),
    ("checksum-error@ddnet.tw", "checksum_error", "vital", "u:id i:error"),
]

SYSTEM_MSGS_0_7 = [
    ( 1, "info", "vital", "s:version s?:password i?:client_version"),
    ( 2, "map_change", "vital", "s:name i:crc i:size i:chunk_num i:chunk_size h:sha256"),
    ( 3, "map_data", "vital", "r:data"),
    ( 4, "server_info", "vital", "r:data"),
    ( 5, "con_ready", "vital", ""),
    ( 6, "snap", "", "i:tick i:delta_tick i:num_parts i:part i:crc d:data"),
    ( 7, "snap_empty", "", "i:tick i:delta_tick"),
    ( 8, "snap_single", "", "i:tick i:delta_tick i:crc d:data"),
    (10, "input_timing", "", "i:input_pred_tick i:time_left"),
    (11, "rcon_auth_on", "vital", ""),
    (12, "rcon_auth_off", "vital", ""),
    (13, "rcon_line", "vital", "s:line"),
    (14, "rcon_cmd_add", "vital", "s:name s:help s:params"),
    (15, "rcon_cmd_rem", "vital", "s:name"),
    (18, "ready", "vital", ""),
    (19, "enter_game", "vital", ""),
    (20, "input", "", "i:ack_snapshot i:intended_tick i:input_size inp:input"),
    (21, "rcon_cmd", "vital", "s:cmd"),
    (22, "rcon_auth", "vital", "s:password"),
    (23, "request_map_data", "vital", ""),
    (26, "ping", "", ""),
    (27, "ping_reply", "", ""),
    (29, "maplist_entry_add", "vital", "s:name"),
    (30, "maplist_entry_rem", "vital", "s:name"),
]


def make_msgs(msgs):
    result = []
    for msg_id, name, vital, members in msgs:
        result_members = []
        for member in members.split():
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
            elif type_ == 'h':
                new_type = datatypes.NetSha256
            elif type_ == 'r':
                new_type = datatypes.NetDataRest
            elif type_ == 'u':
                new_type = datatypes.NetUuid
            elif type_ == 'b':
                new_type = datatypes.NetBool
            elif type_ == 'inp':
                new_type = lambda name: datatypes.NetObjectMember(name, ("player", "input"))
            else:
                raise ValueError("Invalid member: {!r}".format(member))
            member = new_type(member_name)
            if optional:
                member = datatypes.NetOptional(member_name, member)
            result_members.append(member)
        kwargs = {}
        if not isinstance(msg_id, int):
            kwargs["ex"] = msg_id
        result.append(datatypes.NetMessage(name, result_members, **kwargs))

    for (msg_id, _, _, _), struct in zip(msgs, result):
        if not isinstance(msg_id, int):
            msg_id = None
        struct.init(msg_id, [], [], [])

    return result

SYSTEM_MSGS = {
    "0.5": make_msgs(SYSTEM_MSGS_0_5),
    "0.6": make_msgs(SYSTEM_MSGS_0_6),
    "ddnet-15.2.5": make_msgs(SYSTEM_MSGS_DDNET_15_2_5),
    "ddnet-16.2": make_msgs(SYSTEM_MSGS_DDNET_16_2),
    "0.7": make_msgs(SYSTEM_MSGS_0_7),
}
