extern crate arrayvec;
extern crate buffer;
extern crate clap;
extern crate common;
extern crate gamenet_ddnet;
extern crate gamenet_teeworlds_0_7;
extern crate logger;
extern crate packer;
extern crate teehistorian;
extern crate uuid;
extern crate vec_map;
extern crate warn;

use arrayvec::ArrayVec;
use common::num::Cast;
use common::pretty;
use gamenet_ddnet::msg::Game as GameDdnet;
use gamenet_ddnet::msg::game as game_ddnet;
use gamenet_teeworlds_0_7::msg::Game as Game7;
use gamenet_teeworlds_0_7::msg::game as game7;
use packer::Unpacker;
use std::path::Path;
use std::process;
use std::str;
use teehistorian::Buffer;
use teehistorian::Error;
use teehistorian::Item;
use teehistorian::Reader;
use uuid::Uuid;
use vec_map::VecMap;
use warn::Ignore;

#[allow(unused)]
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

fn process(path: &Path) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let (_, mut reader) = Reader::open(path, &mut buffer)?;
    let mut tick = None;
    let mut supplied_infos: VecMap<Info> = VecMap::new();
    let mut ver7: VecMap<bool> = VecMap::new();
    while let Some(item) = reader.read(&mut buffer)? {
        match item {
            Item::TickStart(t) => {
                assert!(tick.is_none());
                tick = Some(t);
            },
            Item::TickEnd(t) => {
                assert_eq!(tick, Some(t));
                tick = None;
            },
            Item::Join(i) => {
                println!("{} player_join cid={}", tick.expect("in tick"), i.cid);
            },
            Item::Drop(i) => {
                println!("{} player_drop cid={}", tick.expect("in tick"), i.cid);
            },
            Item::Joinver6(jv) => {
                ver7.insert(jv.cid.assert_usize(), false);
            },
            Item::Joinver7(jv) => {
                ver7.insert(jv.cid.assert_usize(), true);
            },
            Item::Message(msg) => {
                let mut p = Unpacker::new(msg.msg);
                let mut info: Option<Info> = None;
                if !ver7.get(msg.cid.assert_usize()).cloned().unwrap_or(false) {
                    if let Ok(m) = GameDdnet::decode(&mut Ignore, &mut p) {
                        match m {
                            GameDdnet::ClStartInfo(i) => {
                                info = Some(i.into());
                            },
                            GameDdnet::ClChangeInfo(i) => {
                                info = Some(i.into());
                            },
                            _ => {},
                        }
                    }
                } else {
                    if let Ok(m) = Game7::decode(&mut Ignore, &mut p) {
                        match m {
                            Game7::ClStartInfo(i) => {
                                info = Some(i.into());
                            },
                            _ => {},
                        }
                    }
                }
                if let Some(i) = info {
                    if supplied_infos.get(msg.cid.assert_usize()).map(|prev| prev.name != i.name).unwrap_or(true) {
                        println!("{} player_name cid={} name={:?}", tick.expect("in tick"), msg.cid, pretty::AlmostString::new(&i.name));
                    }
                    supplied_infos.insert(msg.cid.assert_usize(), i);
                }
            }
            Item::PlayerTeam(i) => {
                println!("{} player_team team={} cid={}", tick.expect("in tick"), i.team, i.cid);
            },
            Item::TeamLoadSuccess(i) => {
                let game_uuid = i.save.split(|&b| b == b'\n').nth(1)
                    .and_then(|tee| tee.split(|&b| b == b'\t').nth(100))
                    .and_then(|game_uuid| str::from_utf8(game_uuid).ok())
                    .and_then(|game_uuid| Uuid::parse_str(game_uuid).ok());
                if let Some(u) = game_uuid {
                    println!("{} team_load team={} uuid={} prev_game_uuid={}", tick.expect("in tick"), i.team, i.save_uuid, u);
                } else {
                    println!("{} team_load team={} uuid={}", tick.expect("in tick"), i.team, i.save_uuid);
                }
            },
            Item::TeamSaveSuccess(i) => {
                println!("{} team_save team={} uuid={}", tick.expect("in tick"), i.team, i.save_uuid);
            },
            _ => {},
        }
    }
    assert!(tick.is_none());
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian reader")
        .about("Reads teehistorian file and dumps its contents in a human-readable\
                text stream")
        .arg(Arg::with_name("TEEHISTORIAN")
            .help("Sets the teehistorian file to dump")
            .required(true)
        )
        .get_matches();

    let path = Path::new(matches.value_of_os("TEEHISTORIAN").unwrap());

    match process(path) {
        Ok(()) => {},
        Err(err) => {
            eprintln!("{}: {:?}", path.display(), err);
            process::exit(1);
        }
    }
}
