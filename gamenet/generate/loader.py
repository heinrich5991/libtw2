from datatypes import *
import msg_system

import importlib.util

def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

VERSION_AUTO="auto"
VERSION_0_5="0.5"
VERSION_0_6="0.6"
VERSION_0_7_5="0.7.5"
VERSION_0_7="0.7"
VERSION_DDNET_15_2_5="ddnet-15.2.5"
VERSION_DDNET_16_2="ddnet-16.2"
VERSION_DDNET_16_7_2="ddnet-16.7.2"
VERSION_DDNET_17_2_1="ddnet-17.2.1"
VERSION_DDNET_19_1="ddnet-19.1"
# Version determines how the loaded network files are fixed up. Use `None` to
# disable fixing up.
def load_network(path, version):
    network = load_module("network", path)
    if version is not None:
        fix_network(network, version)
    return network

TUNE_PARAM_NAMES_0_6 = "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking".split()
TUNE_PARAM_NAMES_0_7 = "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost PlayerCollision PlayerHooking".split()

TUNE_PARAM_NAMES = {
    VERSION_0_5: TUNE_PARAM_NAMES_0_6,
    VERSION_0_6: TUNE_PARAM_NAMES_0_6,
    VERSION_0_7_5: TUNE_PARAM_NAMES_0_7,
    VERSION_0_7: TUNE_PARAM_NAMES_0_7,
    VERSION_DDNET_15_2_5: "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking JetpackStrength ShotgunStrength ExplosionStrength HammerStrength HookDuration HammerFireDelay GunFireDelay ShotgunFireDelay GrenadeFireDelay LaserFireDelay NinjaFireDelay".split(),
    VERSION_DDNET_16_2: "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking JetpackStrength ShotgunStrength ExplosionStrength HammerStrength HookDuration HammerFireDelay GunFireDelay ShotgunFireDelay GrenadeFireDelay LaserFireDelay NinjaFireDelay HammerHitFireDelay".split(),
    VERSION_DDNET_16_7_2: "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking JetpackStrength ShotgunStrength ExplosionStrength HammerStrength HookDuration HammerFireDelay GunFireDelay ShotgunFireDelay GrenadeFireDelay LaserFireDelay NinjaFireDelay HammerHitFireDelay".split(),
    VERSION_DDNET_17_2_1: "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking JetpackStrength ShotgunStrength ExplosionStrength HammerStrength HookDuration HammerFireDelay GunFireDelay ShotgunFireDelay GrenadeFireDelay LaserFireDelay NinjaFireDelay HammerHitFireDelay GroundElasticityX GroundElasticityY".split(),
    VERSION_DDNET_19_1: "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking JetpackStrength ShotgunStrength ExplosionStrength HammerStrength HookDuration HammerFireDelay GunFireDelay ShotgunFireDelay GrenadeFireDelay LaserFireDelay NinjaFireDelay HammerHitFireDelay GroundElasticityX GroundElasticityY".split(),
}
MAX_CLIENTS = {
    VERSION_0_5: 16,
    VERSION_0_6: 16,
    VERSION_0_7_5: 64,
    VERSION_0_7: 64,
    VERSION_DDNET_15_2_5: 64,
    VERSION_DDNET_16_2: 64,
    VERSION_DDNET_16_7_2: 64,
    VERSION_DDNET_17_2_1: 64,
    VERSION_DDNET_19_1: 128,
}
NETVERSION = {
    VERSION_0_5: "0.5 b67d1f1a1eea234e",
    VERSION_0_6: "0.6 626fce9a778df4d4",
    VERSION_0_7_5: "0.7 802f1be60a05665f",
    VERSION_0_7: "0.7 802f1be60a05665f",
    VERSION_DDNET_15_2_5: "0.6 626fce9a778df4d4",
    VERSION_DDNET_16_2: "0.6 626fce9a778df4d4",
    VERSION_DDNET_16_7_2: "0.6 626fce9a778df4d4",
    VERSION_DDNET_17_2_1: "0.6 626fce9a778df4d4",
    VERSION_DDNET_19_1: "0.6 626fce9a778df4d4",
}

def fix_network(network, version):
    if version == VERSION_AUTO:
        version = VERSION_0_6
        if any(e.name == ("playerstate",) for e in network.Enums):
            version = VERSION_0_5
        elif any("ddnet" in m.name or "ddrace" in m.name for m in network.Messages):
            version = VERSION_DDNET_15_2_5
            try:
                if len(network.GameInfoFlags2) > 9:
                    version = VERSION_DDNET_19_1
                elif hasattr(network, "LegacyProjectileFlags"):
                    version = VERSION_DDNET_17_2_1
                elif len(network.GameInfoFlags2) > 7:
                    version = VERSION_DDNET_16_7_2
                elif len(network.GameInfoFlags2) > 4:
                    version = VERSION_DDNET_16_2
            except AttributeError:
                pass
        elif "NUM_SKINPARTS" in network.RawHeader:
            version = VERSION_0_7
            if ("spec", "invalidid") in network.GameMsgIDs.values:
                version = VERSION_0_7_5

    network.System = msg_system.SYSTEM_MSGS[version]

    network.Constants = []
    network.Constants += [
        Constant("MAX_CLIENTS", MAX_CLIENTS[version]),
    ]
    network.Constants += [
        Constant("WEAPON_GAME", -3),
        Constant("WEAPON_SELF", -2),
        Constant("WEAPON_WORLD", -1),
    ]
    if version in (VERSION_0_6, VERSION_DDNET_15_2_5, VERSION_DDNET_16_2, VERSION_DDNET_16_7_2, VERSION_DDNET_17_2_1, VERSION_DDNET_19_1):
        network.Constants += [
            Constant("SPEC_FREEVIEW", -1),
        ]
    if version in (VERSION_DDNET_15_2_5, VERSION_DDNET_16_2, VERSION_DDNET_16_7_2, VERSION_DDNET_17_2_1, VERSION_DDNET_19_1):
        network.Constants += [
            Constant("NUM_TUNEZONES", 256),
        ]
    if version != VERSION_0_5:
        network.Constants += [
            Constant("FLAG_MISSING", -3),
            Constant("FLAG_ATSTAND", -2),
            Constant("FLAG_TAKEN", -1),
        ]
    network.Constants += [
        Constant("VERSION", NETVERSION[version]),
    ]
    if version == VERSION_DDNET_15_2_5:
        network.Constants += [
            Constant("DDNET_VERSION", 15025),
        ]
    if version == VERSION_DDNET_16_2:
        network.Constants += [
            Constant("DDNET_VERSION", 16020),
        ]
    if version == VERSION_DDNET_16_7_2:
        network.Constants += [
            Constant("DDNET_VERSION", 16072),
        ]
    if version == VERSION_DDNET_17_2_1:
        network.Constants += [
            Constant("DDNET_VERSION", 17021),
        ]
    if version == VERSION_DDNET_19_1:
        network.Constants += [
            Constant("DDNET_VERSION", 19010),
        ]
    if version == VERSION_0_7_5:
        network.Constants += [
            Constant("CLIENT_VERSION", 0x0705),
        ]
    if version == VERSION_0_7:
        network.Constants += [
            Constant("CLIENT_VERSION", 0x0705),
        ]
    network.Constants += [
        Constant("CL_CALL_VOTE_TYPE_OPTION", "option"),
        Constant("CL_CALL_VOTE_TYPE_KICK", "kick"),
    ]
    if version != VERSION_0_5:
        network.Constants += [
            Constant("CL_CALL_VOTE_TYPE_SPEC", "spec"),
        ]
    if version == VERSION_0_7:
        network.Constants += [
            Constant("VOTE_CHOICE_NO", -1),
            Constant("VOTE_CHOICE_PASS", 0),
            Constant("VOTE_CHOICE_YES", 1),
        ]

    network.Enums += [
        Enum("WEAPON", "HAMMER PISTOL SHOTGUN GRENADE RIFLE NINJA".split()),
    ]
    if version != VERSION_DDNET_19_1:
        network.Enums += [
            Enum("TEAM", "SPECTATORS RED BLUE".split(), offset=-1),
        ]
    if version != VERSION_0_5:
        network.Enums += [
            Enum("SOUND", "GUN_FIRE SHOTGUN_FIRE GRENADE_FIRE HAMMER_FIRE HAMMER_HIT NINJA_FIRE GRENADE_EXPLODE NINJA_HIT RIFLE_FIRE RIFLE_BOUNCE WEAPON_SWITCH PLAYER_PAIN_SHORT PLAYER_PAIN_LONG BODY_LAND PLAYER_AIRJUMP PLAYER_JUMP PLAYER_DIE PLAYER_SPAWN PLAYER_SKID TEE_CRY HOOK_LOOP HOOK_ATTACH_GROUND HOOK_ATTACH_PLAYER HOOK_NOATTACH PICKUP_HEALTH PICKUP_ARMOR PICKUP_GRENADE PICKUP_SHOTGUN PICKUP_NINJA WEAPON_SPAWN WEAPON_NOAMMO HIT CHAT_SERVER CHAT_CLIENT CHAT_HIGHLIGHT CTF_DROP CTF_RETURN CTF_GRAB_PL CTF_GRAB_EN CTF_CAPTURE MENU".split()),
        ]
    else:
        network.Enums += [
            Enum("SOUND", "GUN_FIRE SHOTGUN_FIRE GRENADE_FIRE HAMMER_FIRE HAMMER_HIT NINJA_FIRE GRENADE_EXPLODE NINJA_HIT RIFLE_FIRE RIFLE_BOUNCE WEAPON_SWITCH PLAYER_PAIN_SHORT PLAYER_PAIN_LONG BODY_LAND PLAYER_AIRJUMP PLAYER_JUMP PLAYER_DIE PLAYER_SPAWN PLAYER_SKID TEE_CRY HOOK_LOOP HOOK_ATTACH_GROUND HOOK_ATTACH_PLAYER HOOK_NOATTACH PICKUP_HEALTH PICKUP_ARMOR PICKUP_GRENADE PICKUP_SHOTGUN PICKUP_NINJA WEAPON_SPAWN WEAPON_NOAMMO HIT CHAT_SERVER CHAT_CLIENT CTF_DROP CTF_RETURN CTF_GRAB_PL CTF_GRAB_EN CTF_CAPTURE".split()),
        ]

    if version in (VERSION_0_7_5, VERSION_0_7):
        network.Enums += [
            Enum("SPEC", "FREEVIEW PLAYER FLAGRED FLAGBLUE".split()),
            Enum("SKINPART", "BODY MARKING DECORATION HANDS FEET EYES".split()),
        ]

    TUNE_PARAMS = ("sv", "tune", "params")
    EXTRA_PROJECTILE = ("sv", "extra", "projectile")
    IS_DDNET = ("cl", "is", "ddnet")
    IS_DDNET_LEGACY = ("cl", "is", "ddnet", "legacy")
    TEAMS_STATE = ("sv", "teams", "state")
    TEAMS_STATE_LEGACY = ("sv", "teams", "state", "legacy")
    for i in range(len(network.Messages)):
        if network.Messages[i].name == TUNE_PARAMS:
            network.Messages[i] = NetMessage("SvTuneParams", [NetTuneParam(n) for n in TUNE_PARAM_NAMES[version]])
        elif network.Messages[i].name == EXTRA_PROJECTILE:
            network.Messages[i].values.append(NetObjectMember("projectile", ("projectile",)))
        elif network.Messages[i].name in (IS_DDNET, IS_DDNET_LEGACY):
            network.Messages[i].values.append(NetIntAny("ddnet_version"))
        elif network.Messages[i].name in (TEAMS_STATE, TEAMS_STATE_LEGACY):
            if not network.Messages[i].values:
                network.Messages[i].values.append(NetArray(NetIntRange("teams", 0, MAX_CLIENTS[version] - 1 + 1), MAX_CLIENTS[version]))
    extra_msg_generation = set(v.type_name for m in network.Messages + network.System for v in m.values if isinstance(v, NetObjectMember))
    for i in range(len(network.Objects)):
        if network.Objects[i].name in extra_msg_generation:
            network.Objects[i].attributes.add("msg_encoding")

    network.Connless = []
    if version != VERSION_0_5:
        network.Connless += [
            NetConnless("RequestList", "req2", []),
            NetConnless("List", "lis2", [
                NetAddrs("servers"),
            ]),
            NetConnless("RequestCount", "cou2", []),
            NetConnless("Count", "siz2", [
                NetBigEndianU16("count"),
            ]),
        ]
        if version not in (VERSION_0_7_5, VERSION_0_7):
            network.Connless += [
                NetConnless("RequestInfo", "gie3", [
                    NetU8("token"),
                ]),
            ]
        else:
            network.Connless += [
                NetConnless("RequestInfo", "gie3", [
                    NetIntAny("token"),
                ]),
            ]
        if version in (VERSION_0_6, VERSION_DDNET_15_2_5, VERSION_DDNET_16_2, VERSION_DDNET_16_7_2, VERSION_DDNET_17_2_1, VERSION_DDNET_19_1):
            network.Connless += [
                NetConnless("Info", "inf3", [
                    NetIntString("token"),
                    NetStringStrict("version"),
                    NetStringStrict("name"),
                    NetStringStrict("map"),
                    NetStringStrict("game_type"),
                    NetIntString("flags"),
                    NetIntString("num_players"),
                    NetIntString("max_players"),
                    NetIntString("num_clients"),
                    NetIntString("max_clients"),
                    NetClients("clients"),
                ]),
            ]
        if version in (VERSION_0_7_5, VERSION_0_7):
            network.Connless += [
                NetConnless("Info", "inf3", [
                    NetIntAny("token"),
                    NetStringStrict("version"),
                    NetStringStrict("name"),
                    NetStringStrict("hostname"),
                    NetStringStrict("map"),
                    NetStringStrict("game_type"),
                    NetIntAny("flags"),
                    NetIntRange("skill_level", 0, 2),
                    NetIntAny("num_players"),
                    NetIntAny("max_players"),
                    NetIntAny("num_clients"),
                    NetIntAny("max_clients"),
                    NetClients("clients"),
                ]),
            ]
        if version in (VERSION_DDNET_15_2_5, VERSION_DDNET_16_2, VERSION_DDNET_16_7_2, VERSION_DDNET_17_2_1, VERSION_DDNET_19_1):
            network.Connless += [
                NetConnless("InfoExtended", "iext", [
                    NetIntString("token"),
                    NetStringStrict("version"),
                    NetStringStrict("name"),
                    NetStringStrict("map"),
                    NetIntString("map_crc"),
                    NetIntString("map_size"),
                    NetStringStrict("game_type"),
                    NetIntString("flags"),
                    NetIntString("num_players"),
                    NetIntString("max_players"),
                    NetIntString("num_clients"),
                    NetIntString("max_clients"),
                    NetStringStrict("reserved"),
                    NetClients("clients"),
                ]),
                NetConnless("InfoExtendedMore", "iex+", [
                    NetIntString("token"),
                    NetIntString("packet_no"),
                    NetStringStrict("reserved"),
                    NetClients("clients"),
                ]),
            ]
        network.Connless += [
            NetConnless("Heartbeat", "bea2", [
                NetBigEndianU16("alt_port"),
            ]),
        ]
    network.Connless += [
        NetConnless("ForwardCheck", "fw??", []),
        NetConnless("ForwardResponse", "fw!!", []),
        NetConnless("ForwardOk", "fwok", []),
        NetConnless("ForwardError", "fwer", []),
    ]

    consts = {c.name: c for c in network.Constants}
    enums = {e.name: e for e in network.Enums}
    structs = {s.name: s for s in network.Messages + network.Objects}

    i = 0
    for s in network.Messages:
        index = None
        if s.ex is None:
            index = i + 1
            i += 1
        s.init(index, consts, enums, structs)
    for e in network.Enums:
        e.init(None, consts, enums, structs)
    i = 0
    for o in network.Objects:
        index = None
        if o.ex is None:
            index = i + 1
            i += 1
        o.init(index, consts, enums, structs)
