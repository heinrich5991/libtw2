from collections import namedtuple
import threading

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
    if s.isupper() or s.islower():
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
    def dump(self):
        if self.imports:
            for i in sorted(self.imports):
                _print("use {};".format(i))
            _print()
        _print("\n".join(self.lines))

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
    print("""\
pub const MAX_CLIENTS: i32 = 16;
pub const SPEC_FREEVIEW: i32 = -1;
pub const MAX_SNAPSHOT_PACKSIZE: usize = 900;

pub const FLAG_MISSING: i32 = -3;
pub const FLAG_ATSTAND: i32 = -2;
pub const FLAG_TAKEN: i32 = -1;
""")

def emit_header_snap_obj():
    import_(
        "buffer::CapacityError",
        "enums::Weapon",
        "error::Error",
        "packer::Packer",
        "packer::Unpacker",
        "packer::Warning",
        "warn::Warn",
    )
    print("""\
#[derive(Clone, Copy, Debug)]
pub struct Tick(pub i32);

impl Projectile {
    pub fn decode_msg_inner<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Projectile, Error> {
        Ok(Projectile {
            x: try!(_p.read_int(warn)),
            y: try!(_p.read_int(warn)),
            vel_x: try!(_p.read_int(warn)),
            vel_y: try!(_p.read_int(warn)),
            type_: try!(Weapon::from_i32(try!(_p.read_int(warn)))),
            start_tick: Tick(try!(_p.read_int(warn))),
        })
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        // For the assert!()s.
        self.encode();

        try!(_p.write_int(self.x));
        try!(_p.write_int(self.y));
        try!(_p.write_int(self.vel_x));
        try!(_p.write_int(self.vel_y));
        try!(_p.write_int(self.type_.to_i32()));
        try!(_p.write_int(self.start_tick.0));
        Ok(_p.written())
    }
}

impl PlayerInput {
    pub fn decode_msg_inner<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PlayerInput, Error> {
        Ok(PlayerInput {
            direction: try!(_p.read_int(warn)),
            target_x: try!(_p.read_int(warn)),
            target_y: try!(_p.read_int(warn)),
            jump: try!(_p.read_int(warn)),
            fire: try!(_p.read_int(warn)),
            hook: try!(_p.read_int(warn)),
            player_flags: try!(in_range(try!(_p.read_int(warn)), 0, 256)),
            wanted_weapon: try!(_p.read_int(warn)),
            next_weapon: try!(_p.read_int(warn)),
            prev_weapon: try!(_p.read_int(warn)),
        })
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        // For the assert!()s.
        self.encode();

        try!(_p.write_int(self.direction));
        try!(_p.write_int(self.target_x));
        try!(_p.write_int(self.target_y));
        try!(_p.write_int(self.jump));
        try!(_p.write_int(self.fire));
        try!(_p.write_int(self.hook));
        try!(_p.write_int(self.player_flags));
        try!(_p.write_int(self.wanted_weapon));
        try!(_p.write_int(self.next_weapon));
        try!(_p.write_int(self.prev_weapon));
        Ok(_p.written())
    }
}

pub const PLAYER_INPUT_EMPTY: PlayerInput = PlayerInput {
    direction: 0,
    target_x: 0,
    target_y: 0,
    jump: 0,
    fire: 0,
    hook: 0,
    player_flags: 0,
    wanted_weapon: 0,
    next_weapon: 0,
    prev_weapon: 0,
};
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
impl<'a> Game<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<Game<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::Game(msg_id) =
            SystemOrGame::decode_id(try!(p.read_int(warn)))
        {
            Game::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
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

#[derive(Clone, Copy, Debug)]
pub struct TuneParam(pub i32);

impl TuneParam {
    pub fn from_float(float: f32) -> TuneParam {
        TuneParam((float * 100.0) as i32)
    }
    pub fn to_float(self) -> f32 {
        (self.0 as f32) / 100.0
    }
}
""")

def emit_header_msg_connless():
    import_(
        "buffer::CapacityError",
        "common::pretty",
        "error::Error",
        "packer::Unpacker",
        "packer::Warning",
        "packer::with_packer",
        "std::fmt",
        "super::string_from_int",
        "warn::Warn",
    )
    print("""\
impl<'a> Connless<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Connless<'a>, Error> {
        let id = try!(_p.read_raw(8));
        let connless_id = [id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]];
        Connless::decode_connless(warn, connless_id, _p)
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_raw(&self.connless_id()));
        try!(with_packer(&mut p, |p| self.encode_connless(p)));
        Ok(p.written())
    }
}

pub struct Client<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}

impl<'a> Client<'a> {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.name));
        try!(_p.write_string(self.clan));
        try!(_p.write_string(&string_from_int(self.country)));
        try!(_p.write_string(&string_from_int(self.score)));
        try!(_p.write_string(&string_from_int(self.is_player)));
        Ok(_p.written())
    }
}

impl<'a> fmt::Debug for Client<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Client")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("score", &self.score)
            .field("is_player", &self.is_player)
            .finish()
    }
}

pub const INFO_FLAG_PASSWORD: i32 = 1;
""")

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
        "warn::Warn",
    )
    name = canonicalize(name)
    lifetime = "<'a>" if any(s.lifetime() for s in structs) else ""
    emit_enum_def(name, structs)
    print()
    print("impl{l} {}{l} {{".format(title(name), l=lifetime))
    print("    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: i32, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(title(name), l=lifetime))
    print("        Ok(match msg_id {")
    for s in structs:
        print("            {} => {}::{s}(try!({s}::decode(warn, _p))),".format(caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownId),".format(caps(s.name), title(name), s=title(s.name)))
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
    print("    pub fn decode_obj<W: Warn<ExcessData>>(warn: &mut W, obj_type_id: u16, _p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(name), l=lifetime))
    print("        Ok(match obj_type_id {")
    for s in structs:
        print("            {} => {}::{s}(try!({s}::decode(warn, _p))),".format(caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownId),".format(caps(s.name), title(name), s=title(s.name)))
    print("        })")
    print("    }")
    print("    pub fn obj_type_id(&self) -> u16 {")
    print("        match *self {")
    for s in structs:
        print("            {}::{}(_) => {},".format(title(name), title(s.name), caps(s.name)))
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
        print("            {} => {}::{s}(try!({s}::decode(warn, _p))),".format(caps(s.name), title(name), s=title(s.name)))
    print("            _ => return Err(Error::UnknownId),".format(caps(s.name), title(name), s=title(s.name)))
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
        print("        {} => {},".format(caps(o.name), o.int_size()))
    print("        _ => return None,")
    print("    })")
    print("}")

def emit_enum_module(enums):
    for e in enums:
        e.emit_definition()
        print()
    for e in enums:
        e.emit_impl()
        print()

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
        self.values = [member.update(enums, structs) for member in self.values]

    def emit_consts(self):
        print("pub const {}: {} = {};".format(caps(self.name), self.const_type, self.index))
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
    def emit_impl_encode_decode(self):
        import_(
            "buffer::CapacityError",
            "error::Error",
            "packer::Packer",
            "packer::Unpacker",
            "packer::Warning",
            "std::fmt",
            "warn::Warn",
        )
        print("impl{l} {}{l} {{".format(title(self.name), l=self.lifetime()))
        print("    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
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
        print("    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {{".format(title(self.name), l=self.lifetime()))
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

class NetObject(Struct):
    const_type = "u16"
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
        print("        let result = try!(Self::decode_inner(p));")
        print("        p.finish(warn);")
        print("        Ok(result)")
        print("    }")
        print("    pub fn decode_inner(_p: &mut IntUnpacker{l}) -> Result<{}{l}, Error> {{".format(title(self.name), l=self.lifetime()))
        if self.values or super:
            print("        Ok({} {{".format(title(self.name)))
            if super:
                print("            {}: try!({}::decode_inner(_p)),".format(snake(super.name), title(super.name), super.lifetime()))
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

class NetEvent(NetObject):
    pass

class NetMessage(Struct):
    const_type = "i32"

class NetConnless(Struct):
    def __init__(self, name, id, values):
        super().__init__(name, values)
        self.id = id
    def emit_consts(self):
        print(r"""pub const {}: &'static [u8; 8] = b"\xff\xff\xff\xff{}";""".format(caps(self.name), self.id))

class Member:
    def __init__(self, name):
        self.name = canonicalize(name)
    def definition(self):
        return "{}: {}".format(snake(self.name), self.type_)
    def contains_lifetime(self):
        return "'a" in self.type_
    def update(self, enums, structs):
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
        print("try!({});".format(self.encode_expr("self.{}".format(snake(self.name)))))
    def emit_debug(self):
        print(".field(\"{}\", &{})".format(snake(self.name), self.debug_expr("self.{}".format(snake(self.name)))))
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
        return "[\n{}]".format("".join(
            "    {},\n".format(self.inner.decode_expr()) for _ in range(self.count)
        ))
    def emit_assert(self):
        assert_expr = self.inner.assert_expr("e")
        if assert_expr:
            print("for e in &self.{} {{".format(snake(self.name)))
            print("    {};".format(assert_expr))
            print("}")
    def emit_encode(self):
        print("for e in &self.{} {{".format(snake(self.name)))
        print("    try!({});".format(self.inner.encode_expr("e")))
        print("}")
    def decode_int_expr(self):
        return "[\n{}]".format("".join(
            "    {},\n".format(self.inner.decode_int_expr()) for _ in range(self.count)
        ))
    def debug_expr(self, self_expr):
        import_("debug::DebugSlice")
        return "DebugSlice::new(&{}, |e| {})".format(self_expr, self.inner.debug_expr("e"))
    def int_size(self):
        return self.inner.int_size() * self.count

class NetOptional(Member):
    def __init__(self, name, inner):
        super().__init__(name)
        self.inner = inner
        self.type_ = "Option<{}>".format(inner.type_)
    def decode_expr(self):
        START="try!("
        END=")"
        inner_decode = self.inner.decode_expr()
        if not inner_decode.startswith(START) or not inner_decode.endswith(END):
            raise ValueError("can't form an optional of this type")
        return "{}.ok()".format(inner_decode[len(START):-len(END)])
    def encode_expr(self, self_expr):
        return self.inner.encode_expr("{}.unwrap()").format(self_expr)
    def debug_expr(self, self_expr):
        return "{}.as_ref().map(|v| {})".format(self_expr, self.inner.debug_expr("v"))
    def assert_expr(self, self_expr):
        return "assert!({}.is_some())".format(self_expr)

class NetString(Member):
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "try!(_p.read_string())"
    def encode_expr(self, self_expr):
        return "_p.write_string({})".format(self_expr)
    def debug_expr(self, self_expr):
        import_("common::pretty")
        return "pretty::Bytes::new(&{})".format(self_expr)

class NetStringStrict(NetString):
    def decode_expr(self):
        import_("packer::sanitize")
        return "try!(sanitize(warn, {}))".format(super().decode_expr())
    def assert_expr(self, self_expr):
        import_(
            "packer::sanitize",
            "warn::Panic",
        )
        return "sanitize(&mut Panic, {}).unwrap()".format(self_expr)

class NetData(Member):
    type_ = "&'a [u8]"
    def decode_expr(self):
        return "try!(_p.read_data(warn))"
    def encode_expr(self, self_expr):
        return "_p.write_data({})".format(self_expr)
    def debug_expr(self, self_expr):
        import_("common::pretty")
        return "pretty::Bytes::new(&{})".format(self_expr)

class NetIntAny(Member):
    type_ = "i32"
    def decode_expr(self):
        return "try!(_p.read_int(warn))"
    def encode_expr(self, self_expr):
        return "_p.write_int({})".format(self_expr)
    def decode_int_expr(self):
        return "try!(_p.read_int())"
    def int_size(self):
        return 1

def import_consts(value):
    value = str(value)
    for const in "FLAG_MISSING MAX_CLIENTS SPEC_FREEVIEW TEAM_BLUE TEAM_RED".split():
        if const in value:
            import_("enums::{}".format(const))

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
        if min == "TEAM_SPECTATORS" and max == "TEAM_BLUE":
            return NetEnum(self.name, "team")
        if self.name == ("hooked", "player") and min == "0":
            self.min = -1
        elif self.name == ("emote",) and max == str(len(enums[("emote",)].values)):
            max = "NUM_EMOTES-1"
        if str(self.min) == "0" and max.startswith("NUM_") and max.endswith("S-1"):
            return NetEnum(self.name, max[4:-3])
        if max == "NUM_WEAPONS-1":
            self.max = len(enums[("weapon",)].values) - 1
        return self
    def decode_expr(self):
        import_("packer::in_range")
        import_consts(self.min)
        import_consts(self.max)
        return "try!(in_range({}, {}, {}))".format(super().decode_expr(), self.min, self.max)
    def assert_expr(self, self_expr):
        import_consts(self.min)
        import_consts(self.max)
        return "assert!({} <= {s} && {s} <= {})".format(self.min, self.max, s=self_expr)
    def decode_int_expr(self):
        import_("packer::in_range")
        import_consts(self.min)
        import_consts(self.max)
        return "try!(in_range({}, {}, {}))".format(super().decode_int_expr(), self.min, self.max)

class NetIntPositive(NetIntAny):
    def __init__(self, name):
        super().__init__(name)
    def update(self, enums, structs):
        return self
    def decode_expr(self):
        import_("packer::positive")
        return "try!(positive({}))".format(super().decode_expr())
    def assert_expr(self, self_expr):
        return "assert!({} >= 0)".format(self_expr)
    def decode_int_expr(self):
        import_("packer::positive")
        return "try!(positive({}))".format(super().decode_int_expr())

class NetEnum(NetIntAny):
    def __init__(self, name, enum_name):
        super().__init__(name)
        self.enum_name = canonicalize(enum_name)
        self.type_ = title(self.enum_name)
    def decode_expr(self):
        import_("enums::{}".format(title(self.enum_name)))
        return "try!({}::from_i32({}))".format(title(self.enum_name), super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.to_i32()".format(self_expr))
    def decode_int_expr(self):
        import_("enums::{}".format(title(self.enum_name)))
        return "try!({}::from_i32({}))".format(title(self.enum_name), super().decode_int_expr())

class NetBool(NetIntAny):
    type_ = "bool"
    def decode_expr(self):
        import_("packer::to_bool")
        return "try!(to_bool({}))".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{} as i32".format(self_expr))

class NetTuneParam(NetIntAny):
    type_ = "TuneParam"
    def decode_expr(self):
        return "TuneParam({})".format(super().decode_expr())
    def encode_expr(self, self_expr):
        return super().encode_expr("{}.0".format(self_expr))

class NetTick(NetIntAny):
    type_ = "Tick"
    def decode_int_expr(self):
        return "Tick({})".format(super().decode_int_expr())

class NetStruct(Member):
    def __init__(self, name, type_):
        super().__init__(name)
        self.type_ = type_
    def decode_expr(self):
        return "try!({}::decode_msg_inner(warn, _p))".format(self.type_)
    def encode_expr(self, self_expr):
        import_("packer::with_packer")
        return "with_packer(&mut _p, |p| {}.encode_msg(p))".format(self_expr)

class NetAddrs(Member):
    type_ = "&'a [AddrPacked]"
    def definition(self):
        import_("super::AddrPacked")
        return super().definition()
    def decode_expr(self):
        import_(
            "super::AddrPackedSliceExt",
            "warn::wrap",
        )
        return "AddrPackedSliceExt::from_bytes(wrap(warn), try!(_p.read_rest()))"
    def encode_expr(self, self_expr):
        return "_p.write_rest({}.as_bytes())".format(self_expr)

class NetBigEndianU16(Member):
    type_ = "u16"
    def decode_expr(self):
        import_("common::num::BeU16")
        return "{ let s = try!(_p.read_raw(2)); BeU16::from_bytes(&[s[0], s[1]]).to_u16() }"
    def encode_expr(self, self_expr):
        import_("common::num::BeU16")
        return "_p.write_raw(BeU16::from_u16({}).as_bytes())".format(self_expr)

class NetU8(Member):
    type_ = "u8"
    def decode_expr(self):
        return "try!(_p.read_raw(1))[0]"
    def encode_expr(self, self_expr):
        return "_p.write_raw(&[{}])".format(self_expr)

class NetIntString(NetString):
    type_ = "i32"
    def decode_expr(self):
        import_("super::int_from_string")
        return "try!(int_from_string(try!(_p.read_string())))"
    def encode_expr(self, self_expr):
        import_("super::string_from_int")
        return "_p.write_string(&string_from_int({}))".format(self_expr)
    def debug_expr(self, self_expr):
        return self_expr

class NetClients(Member):
    type_ = "ClientsData<'a>"
    def definition(self):
        import_("super::ClientsData")
        return super().definition()
    def decode_expr(self):
        import_("super::ClientsData")
        return "ClientsData::from_bytes(try!(_p.read_rest()))"
    def encode_expr(self, self_expr):
        return "_p.write_rest({}.as_bytes())".format(self_expr)
