extern crate arrayvec;
extern crate common;
extern crate clap;
extern crate demo;
extern crate gamenet_ddnet;
extern crate gamenet_teeworlds_0_7;
extern crate logger;
extern crate packer;
extern crate snapshot;
extern crate teehistorian;
extern crate vec_map;
extern crate warn;
extern crate world;

use arrayvec::ArrayVec;
use common::num::Cast;
use demo::Writer;
use gamenet_ddnet::enums::Emote;
use gamenet_ddnet::enums::Team;
use gamenet_ddnet::enums::VERSION;
use gamenet_ddnet::enums::Weapon;
use gamenet_ddnet::msg::Game as GameDdnet;
use gamenet_ddnet::msg::game as game_ddnet;
use gamenet_ddnet::snap_obj::PlayerInput;
use gamenet_ddnet::snap_obj;
use gamenet_teeworlds_0_7::msg::Game as Game7;
use gamenet_teeworlds_0_7::msg::game as game7;
use packer::IntUnpacker;
use packer::Unpacker;
use packer::string_to_ints3;
use packer::string_to_ints4;
use packer::string_to_ints6;
use packer::with_packer;
use snapshot::snap::MAX_SNAPSHOT_SIZE;
use snapshot::snap;
use std::ffi::OsString;
use std::path::Path;
use std::process;
use teehistorian::Buffer;
use teehistorian::Error;
use teehistorian::Item;
use teehistorian::Pos;
use teehistorian::Reader;
use vec_map::VecMap;
use warn::Ignore;
use world::vec2;

const TICKS_PER_SECOND: i32 = 50;

struct Info {
    name: ArrayVec<[u8; 4*4-1]>,
    clan: ArrayVec<[u8; 3*4-1]>,
    country: i32,
    skin: ArrayVec<[u8; 6*4-1]>,
    use_custom_color: bool,
    color_body: i32,
    color_feet: i32,
}

impl<'a> From<game_ddnet::ClChangeInfo<'a>> for Info {
    fn from(m: game_ddnet::ClChangeInfo) -> Info {
        Info {
            name: m.name.iter().cloned().collect(),
            clan: m.clan.iter().cloned().collect(),
            country: m.country,
            skin: m.skin.iter().cloned().collect(),
            use_custom_color: m.use_custom_color,
            color_body: m.color_body,
            color_feet: m.color_feet,
        }
    }
}

impl<'a> From<game_ddnet::ClStartInfo<'a>> for Info {
    fn from(m: game_ddnet::ClStartInfo) -> Info {
        Info {
            name: m.name.iter().cloned().collect(),
            clan: m.clan.iter().cloned().collect(),
            country: m.country,
            skin: m.skin.iter().cloned().collect(),
            use_custom_color: m.use_custom_color,
            color_body: m.color_body,
            color_feet: m.color_feet,
        }
    }
}

impl<'a> From<game7::ClStartInfo<'a>> for Info {
    fn from(m: game7::ClStartInfo) -> Info {
        Info {
            name: m.name.iter().cloned().collect(),
            clan: m.clan.iter().cloned().collect(),
            country: m.country,
            skin: b"default".iter().cloned().collect(),
            use_custom_color: false,
            color_body: 0,
            color_feet: 0,
        }
    }
}

fn process(in_: &Path, out: &Path) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let mut snap_buffer = Vec::new();
    let mut th;
    let mut demo;
    {
        let (header, teehistorian) = Reader::open(in_, &mut buffer)?;
        th = teehistorian;
        if let Some(map_sha256) = header.map_sha256 {
            demo = Writer::create_ddnet(
                out,
                VERSION.as_bytes(),
                header.map_name.as_bytes(),
                map_sha256,
                header.map_crc,
                demo::format::TYPE_SERVER,
                b"", // Timestamp
            )?;
        } else {
            demo = Writer::create(
                out,
                VERSION.as_bytes(),
                header.map_name.as_bytes(),
                header.map_crc,
                demo::format::TYPE_SERVER,
                b"", // Timestamp
            )?;
        }
    }
    let mut delta = snap::Delta::new();
    let mut last_full_snap_tick = None;
    let mut last_snap = None;
    let mut builder = snap::Builder::new();
    let mut last_tick = 0;
    let mut ver7: VecMap<bool> = VecMap::new();
    let mut supplied_infos: VecMap<Info> = VecMap::new();
    let mut inputs: VecMap<PlayerInput> = VecMap::new();
    let mut prev_pos: VecMap<Pos> = VecMap::new();
    let mut encoded: Vec<u8> = Vec::with_capacity(MAX_SNAPSHOT_SIZE);
    while let Some(item) = th.read(&mut buffer)? {
        let mut do_ticks = 0..0;
        match item {
            Item::TickStart(tick) => {
                do_ticks = last_tick+1..tick;
            },
            Item::TickEnd(tick) => {
                last_tick = tick;
                do_ticks = tick..tick+1;
            },
            Item::Input(input) => {
                if let Ok(pi) = PlayerInput::decode(&mut Ignore, &mut IntUnpacker::new(&input.input)) {
                    inputs.insert(input.cid.assert_usize(), pi);
                }
            },
            Item::Joinver6(jv) => {
                ver7.insert(jv.cid.assert_usize(), false);
            },
            Item::Joinver7(jv) => {
                ver7.insert(jv.cid.assert_usize(), true);
            },
            Item::Message(msg) => {
                let mut p = Unpacker::new(msg.msg);
                if !ver7.get(msg.cid.assert_usize()).cloned().unwrap_or(false) {
                    if let Ok(m) = GameDdnet::decode(&mut Ignore, &mut p) {
                        match m {
                            GameDdnet::ClStartInfo(i) => {
                                supplied_infos.insert(msg.cid.assert_usize(), i.into());
                            },
                            GameDdnet::ClChangeInfo(i) => {
                                supplied_infos.insert(msg.cid.assert_usize(), i.into());
                            },
                            _ => {},
                        }
                    }
                } else {
                    if let Ok(m) = Game7::decode(&mut Ignore, &mut p) {
                        match m {
                            Game7::ClStartInfo(i) => {
                                supplied_infos.insert(msg.cid.assert_usize(), i.into());
                            },
                            _ => {},
                        }
                    }
                }
            }
            _ => {},
        }
        for tick in do_ticks {
            for cid in th.cids() {
                let maybe_pos = th.player_pos(cid);
                if let Some(pos) = maybe_pos {
                    let ppos = prev_pos.get(cid.assert_usize()).cloned().unwrap_or(pos);
                    let default = PlayerInput::default();
                    let input = inputs.get(cid.assert_usize()).unwrap_or(&default);
                    let info = &supplied_infos[cid.assert_usize()];
                    let name: &[u8] = if !info.name.is_empty() {
                        &info.name
                    } else {
                        // Theoretically we have to track all the names. We
                        // don't do that, so just pretend we care and do the
                        // common case.
                        b"(1)"
                    };
                    let client_info = snap_obj::ClientInfo {
                        name: string_to_ints4(name),
                        clan: string_to_ints3(&info.clan),
                        country: info.country,
                        skin: string_to_ints6(&info.skin),
                        use_custom_color: info.use_custom_color as i32,
                        color_body: info.color_body,
                        color_feet: info.color_feet,
                    };
                    let player_info = snap_obj::PlayerInfo {
                        local: 0,
                        client_id: cid,
                        team: Team::Red,
                        score: -9999,
                        latency: 0,
                    };
                    let target = vec2::new(input.target_x as f32, input.target_y as f32);
                    let character = snap_obj::Character {
                        character_core: snap_obj::CharacterCore {
                            tick: tick,
                            x: pos.x,
                            y: pos.y,
                            vel_x: pos.x - ppos.x,
                            vel_y: pos.y - ppos.y,
                            angle: target.angle().to_net(),
                            direction: input.direction,
                            jumped: (input.jump != 0) as i32,
                            hooked_player: 0,
                            hook_state: -1,
                            hook_tick: snap_obj::Tick(0),
                            hook_x: 0,
                            hook_y: 0,
                            hook_dx: 0,
                            hook_dy: 0,
                        },
                        player_flags: snap_obj::PLAYERFLAG_PLAYING,
                        health: 10,
                        armor: 10,
                        ammo_count: 0,
                        weapon: Weapon::Hammer,
                        emote: Emote::Normal,
                        attack_tick: 0,
                    };
                    builder.add_item(snap_obj::CLIENT_INFO, cid.assert_u16(), client_info.encode()).unwrap();
                    builder.add_item(snap_obj::PLAYER_INFO, cid.assert_u16(), player_info.encode()).unwrap();
                    builder.add_item(snap_obj::CHARACTER, cid.assert_u16(), character.encode()).unwrap();

                    prev_pos.insert(cid.assert_usize(), pos);
                } else {
                    prev_pos.remove(cid.assert_usize());
                }
            }
            let game_info = snap_obj::GameInfo {
                game_flags: 0,
                game_state_flags: 0,
                round_start_tick: snap_obj::Tick(0),
                warmup_timer: 0,
                score_limit: 0,
                time_limit: 0,
                round_num: 0,
                round_current: 1,
            };
            builder.add_item(snap_obj::GAME_INFO, 0, game_info.encode()).unwrap();
            let snap = builder.finish();

            encoded.clear();
            match (&last_snap, last_full_snap_tick) {
                (&Some(ref l), Some(t)) if tick - t <= 5 * TICKS_PER_SECOND => {
                    demo.write_tick(false, demo::Tick(tick))?;
                    delta.create(l, &snap);
                    demo.write_snapshot_delta(with_packer(&mut encoded, |p| {
                        delta.write(snap_obj::obj_size, p).unwrap()
                    }))?;
                },
                _ => {
                    demo.write_tick(true, demo::Tick(tick))?;
                    demo.write_snapshot(with_packer(&mut encoded, |p| {
                        snap.write(&mut snap_buffer, p).unwrap()
                    }))?;
                    last_full_snap_tick = Some(tick);
                }
            }
            if let Some(l) = last_snap {
                builder = l.recycle();
            } else {
                builder = snap::Builder::new();
            }
            last_snap = Some(snap);
        }
    }
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian to demo converter")
        .about("Converts teehistorian data to a demo file.")
        .arg(Arg::with_name("TEEHISTORIAN")
            .help("Sets the input teehistorian file")
            .required(true)
        )
        .arg(Arg::with_name("DEMO")
            .help("Sets the output demo file")
        )
        .get_matches();

    let mut buffer;
    let in_ = Path::new(matches.value_of_os("TEEHISTORIAN").unwrap());
    let out = match matches.value_of_os("DEMO").map(Path::new) {
        Some(o) => o,
        None => {
            buffer = OsString::from(in_);
            buffer.push(".demo");
            Path::new(&buffer)
        },
    };

    match process(in_, out) {
        Ok(()) => {},
        Err(err) => {
            println!("{}: {:?}", in_.display(), err);
            process::exit(1);
        }
    }
}
