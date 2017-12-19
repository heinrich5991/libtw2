extern crate arrayvec;
extern crate common;
extern crate clap;
extern crate demo;
extern crate gamenet;
extern crate logger;
extern crate packer;
extern crate snapshot;
extern crate teehistorian;
extern crate vec_map;
extern crate void;
extern crate warn;

use arrayvec::ArrayVec;
use common::num::Cast;
use demo::Writer;
use gamenet::enums::Emote;
use gamenet::enums::Team;
use gamenet::enums::Weapon;
use gamenet::msg::Game;
use gamenet::msg::game;
use gamenet::snap_obj;
use packer::Unpacker;
use packer::string_to_ints3;
use packer::string_to_ints4;
use packer::string_to_ints6;
use snapshot::snap;
use std::path::Path;
use std::process;
use teehistorian::Buffer;
use teehistorian::Error;
use teehistorian::Item;
use teehistorian::Reader;
use vec_map::VecMap;
use void::ResultVoidExt;
use void::Void;
use warn::Ignore;

struct Info {
    name: ArrayVec<[u8; 4*4-1]>,
    clan: ArrayVec<[u8; 3*4-1]>,
    country: i32,
    skin: ArrayVec<[u8; 6*4-1]>,
    use_custom_color: bool,
    color_body: i32,
    color_feet: i32,
}

impl<'a> From<game::ClChangeInfo<'a>> for Info {
    fn from(m: game::ClChangeInfo) -> Info {
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

impl<'a> From<game::ClStartInfo<'a>> for Info {
    fn from(m: game::ClStartInfo) -> Info {
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

fn process(in_: &Path, out: &Path) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let mut snap_buffer = Vec::new();
    let mut snap_repr = Vec::new();
    let mut th = Reader::open(in_, &mut buffer)?;
    let mut demo = Writer::create(
        out,
        gamenet::VERSION,
        b"GetSpeed",
        0xd46ed0aa,
        demo::format::TYPE_SERVER,
        b"", // Timestamp
    )?;
    let mut builder = snap::Builder::new();
    let mut last_tick = 0;
    let mut supplied_infos: VecMap<Info> = VecMap::new();
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
            Item::Message(msg) => {
                let mut p = Unpacker::new(msg.msg);
                if let Ok(m) = Game::decode(&mut Ignore, &mut p) {
                    match m {
                        Game::ClStartInfo(i) => {
                            supplied_infos.insert(msg.cid.assert_usize(), i.into());
                        },
                        Game::ClChangeInfo(i) => {
                            supplied_infos.insert(msg.cid.assert_usize(), i.into());
                        },
                        _ => {},
                    }
                }
            }
            _ => {},
        }
        for tick in do_ticks {
            for cid in th.cids() {
                if let Some(pos) = th.player_pos(cid) {
                    let info = &supplied_infos[cid.assert_usize()];
                    let name: &[u8] = if !info.name.is_empty() {
                        &info.name
                    } else {
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
                        score: 0,
                        latency: 0,
                    };
                    let character = snap_obj::Character {
                        character_core: snap_obj::CharacterCore {
                            tick: tick,
                            x: pos.x,
                            y: pos.y,
                            vel_x: 0,
                            vel_y: 0,
                            angle: 0,
                            direction: 0,
                            jumped: 0,
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
            demo.write_tick(true, demo::Tick(tick))?;
            snap_repr.clear();
            snap.write_full(|s| -> Result<(), Void> { Ok(snap_repr.extend(s)) }, &mut snap_buffer).void_unwrap();
            demo.write_snapshot(&snap_repr)?;
            builder = snap.recycle();
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
            .required(true)
        )
        .get_matches();

    let in_ = Path::new(matches.value_of_os("TEEHISTORIAN").unwrap());
    let out = Path::new(matches.value_of_os("DEMO").unwrap());

    match process(in_, out) {
        Ok(()) => {},
        Err(err) => {
            println!("{}: {:?}", in_.display(), err);
            process::exit(1);
        }
    }
}
