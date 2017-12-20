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
extern crate world;

use arrayvec::ArrayVec;
use common::num::Cast;
use demo::Writer;
use gamenet::enums::Emote;
use gamenet::enums::Team;
use gamenet::enums::Weapon;
use gamenet::msg::Game;
use gamenet::msg::game;
use gamenet::snap_obj::PLAYER_INPUT_EMPTY;
use gamenet::snap_obj::PlayerInput;
use gamenet::snap_obj;
use packer::IntUnpacker;
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
use teehistorian::Pos;
use teehistorian::Reader;
use vec_map::VecMap;
use void::ResultVoidExt;
use void::Void;
use warn::Ignore;
use world::vec2;

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
    let mut th;
    let mut demo;
    {
        let (header, teehistorian) = Reader::open(in_, &mut buffer)?;
        th = teehistorian;
        demo = Writer::create(
            out,
            gamenet::VERSION,
            header.map_name.as_bytes(),
            header.map_crc,
            demo::format::TYPE_SERVER,
            b"", // Timestamp
        )?;
    }
    let mut builder = snap::Builder::new();
    let mut last_tick = 0;
    let mut supplied_infos: VecMap<Info> = VecMap::new();
    let mut inputs: VecMap<PlayerInput> = VecMap::new();
    let mut prev_pos: VecMap<Pos> = VecMap::new();
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
                let maybe_pos = th.player_pos(cid);
                if let Some(pos) = maybe_pos {
                    let ppos = prev_pos.get(cid.assert_usize()).cloned().unwrap_or(pos);
                    let input = inputs.get(cid.assert_usize()).unwrap_or(&PLAYER_INPUT_EMPTY);
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
            demo.write_tick(true, demo::Tick(tick))?;
            snap_repr.clear();
            snap.write_full(|s| -> Result<(), Void> { Ok(snap_repr.extend(s)) }, &mut snap_buffer).void_unwrap();
            // TODO: Write deltas…
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
