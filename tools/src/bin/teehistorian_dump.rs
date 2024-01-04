extern crate buffer;
extern crate clap;
extern crate gamenet_teeworlds_0_6 as gamenet;
extern crate logger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate teehistorian;
extern crate warn;

use std::io;
use std::path::Path;
use std::process;
use teehistorian::Buffer;
use teehistorian::Error;
use teehistorian::Item;
use teehistorian::Reader;

#[derive(Serialize)]
struct TickAndItem<'a> {
    tick: i32,
    item: Item<'a>,
}

fn process(path: &Path, json: bool) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let (_, mut reader) = Reader::open(path, &mut buffer)?;
    let mut tick = None;
    if json {
        println!("[");
    }
    let mut first = true;
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
            _ => {
                if !first {
                    if json {
                        println!(",");
                    }
                } else {
                    first = false;
                }
                if json {
                    let stdout = io::stdout();
                    serde_json::to_writer(
                        stdout.lock(),
                        &TickAndItem {
                            tick: tick.unwrap(),
                            item: item,
                        },
                    )
                    .unwrap();
                } else {
                    println!("{} {:?}", tick.expect("in tick"), item);
                }
            }
        }
    }
    assert!(tick.is_none());
    if json {
        println!();
        println!("]");
    }
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian reader")
        .about(
            "Reads teehistorian file and dumps its contents in a human-readable\
                text stream",
        )
        .arg(
            Arg::with_name("TEEHISTORIAN")
                .help("Sets the teehistorian file to dump")
                .required(true),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .help("Output machine-readable JSON"),
        )
        .get_matches();

    let path = Path::new(matches.value_of_os("TEEHISTORIAN").unwrap());
    let json = matches.is_present("json");

    match process(path, json) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}: {:?}", path.display(), err);
            process::exit(1);
        }
    }
}
