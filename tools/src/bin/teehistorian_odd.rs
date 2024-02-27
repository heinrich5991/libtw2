extern crate gamenet_teeworlds_0_6 as gamenet;

use arrayvec::ArrayVec;
use common::num::Cast;
use common::pretty;
use gamenet::msg::game;
use gamenet::msg::Game;
use packer::Unpacker;
use std::path::Path;
use std::process;
use teehistorian::format::item::INPUT_LEN;
use teehistorian::Buffer;
use teehistorian::Error;
use teehistorian::Input;
use teehistorian::Item;
use teehistorian::Reader;
use vec_map::VecMap;
use warn::Ignore;

struct Info {
    name: ArrayVec<[u8; 4 * 4 - 1]>,
}

impl<'a> From<game::ClChangeInfo<'a>> for Info {
    fn from(m: game::ClChangeInfo) -> Info {
        Info {
            name: m.name.iter().cloned().collect(),
        }
    }
}

impl<'a> From<game::ClStartInfo<'a>> for Info {
    fn from(m: game::ClStartInfo) -> Info {
        Info {
            name: m.name.iter().cloned().collect(),
        }
    }
}

struct PrevInput {
    tick: i32,
    input: [i32; INPUT_LEN],
}

const ODD: i32 = 10;
const FIRE: usize = 4;
const INPUT_STATE_MASK: i32 = 0x3f;
const TICKS_PER_SECOND: i32 = 50;

fn process(path: &Path) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let (_, mut reader) = Reader::open(path, &mut buffer)?;
    let mut tick = None;
    let mut inputs: VecMap<PrevInput> = VecMap::new();
    let mut infos: VecMap<Info> = VecMap::new();
    while let Some(item) = reader.read(&mut buffer)? {
        match item {
            Item::TickStart(t) => {
                assert!(tick.is_none());
                tick = Some(t);
            }
            Item::TickEnd(t) => {
                assert_eq!(tick, Some(t));
                tick = None;
            }
            Item::Input(Input { cid, input }) => {
                let name = pretty::AlmostString::new(&infos[cid.assert_usize()].name);
                let tick = tick.expect("in tick");
                if let Some(prev_input) = inputs.get(cid.assert_usize()) {
                    let prev_fire = prev_input.input[FIRE] & INPUT_STATE_MASK;
                    let fire = input[FIRE] & INPUT_STATE_MASK;
                    let df = (fire + INPUT_STATE_MASK + 1 - prev_fire) & INPUT_STATE_MASK;
                    if df > ODD {
                        let clicks = df / 2 * TICKS_PER_SECOND * 10;
                        let dt = tick - prev_input.tick;
                        if dt != 0 {
                            let cps = clicks / dt;
                            println!(
                                "name={:?} dt={} df={} cps={}.{}",
                                name,
                                dt,
                                df,
                                cps / 10,
                                cps % 10
                            );
                        } else {
                            println!("name={:?} dt={} df={} cps=nan", name, dt, df);
                        }
                    }
                    if input[FIRE] > INPUT_STATE_MASK {
                        //println!("weird fire name={:?} t={} f={}", name, tick, input[FIRE]);
                    }
                }
                inputs.insert(
                    cid.assert_usize(),
                    PrevInput {
                        tick: tick,
                        input: input,
                    },
                );
            }
            Item::Message(msg) => {
                let mut p = Unpacker::new(msg.msg);
                if let Ok(m) = Game::decode(&mut Ignore, &mut p) {
                    match m {
                        Game::ClStartInfo(i) => {
                            infos.insert(msg.cid.assert_usize(), i.into());
                        }
                        Game::ClChangeInfo(i) => {
                            infos.insert(msg.cid.assert_usize(), i.into());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    assert!(tick.is_none());
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian odd input checker")
        .about("Reads teehistorian file and checks for odd inputs")
        .arg(
            Arg::with_name("TEEHISTORIAN")
                .help("Sets the teehistorian file to search")
                .required(true),
        )
        .get_matches();

    let path = Path::new(matches.value_of_os("TEEHISTORIAN").unwrap());

    match process(path) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}: {:?}", path.display(), err);
            process::exit(1);
        }
    }
}
