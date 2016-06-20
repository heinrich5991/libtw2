import datatypes
from datatypes import *
import network

network.Enums += [
    Enum("WEAPON", "HAMMER PISTOL SHOTGUN GRENADE RIFLE NINJA".split()),
    Enum("TEAM", "SPECTATORS RED BLUE".split(), offset=-1),
    Enum("SOUND", "GUN_FIRE SHOTGUN_FIRE GRENADE_FIRE HAMMER_FIRE HAMMER_HIT NINJA_FIRE GRENADE_EXPLODE NINJA_HIT RIFLE_FIRE RIFLE_BOUNCE WEAPON_SWITCH PLAYER_PAIN_SHORT PLAYER_PAIN_LONG BODY_LAND PLAYER_AIRJUMP PLAYER_JUMP PLAYER_DIE PLAYER_SPAWN PLAYER_SKID TEE_CRY HOOK_LOOP HOOK_ATTACH_GROUND HOOK_ATTACH_PLAYER HOOK_NOATTACH PICKUP_HEALTH PICKUP_ARMOR PICKUP_GRENADE PICKUP_SHOTGUN PICKUP_NINJA WEAPON_SPAWN WEAPON_NOAMMO HIT CHAT_SERVER CHAT_CLIENT CHAT_HIGHLIGHT CTF_DROP CTF_RETURN CTF_GRAB_PL CTF_GRAB_EN CTF_CAPTURE MENU".split()),
]

TUNE_PARAMS = ("sv", "tune", "params")
for i in range(len(network.Messages)):
    if network.Messages[i].name == TUNE_PARAMS:
        network.Messages[i] = NetMessage("SvTuneParams", [NetIntAny(n) for n in "GroundControlSpeed GroundControlAccel GroundFriction GroundJumpImpulse AirJumpImpulse AirControlSpeed AirControlAccel AirFriction HookLength HookFireSpeed HookDragAccel HookDragSpeed Gravity VelrampStart VelrampRange VelrampCurvature GunCurvature GunSpeed GunLifetime ShotgunCurvature ShotgunSpeed ShotgunSpeeddiff ShotgunLifetime GrenadeCurvature GrenadeSpeed GrenadeLifetime LaserReach LaserBounceDelay LaserBounceNum LaserBounceCost LaserDamage PlayerCollision PlayerHooking".split()])
        break

enums = {e.name: e for e in network.Enums}
structs = {s.name: s for s in network.Messages + network.Objects}

for i, s in enumerate(network.Messages):
    s.init(i + 1, enums, structs)
for e in network.Enums:
    e.init(None, enums, structs)
for i, o in enumerate(network.Objects):
    o.init(i + 1, enums, structs)
