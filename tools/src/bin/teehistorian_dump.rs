extern crate buffer;
extern crate clap;
extern crate gamenet;
extern crate logger;
extern crate teehistorian;
extern crate warn;

use std::path::Path;
use std::process;
use teehistorian::Buffer;
use teehistorian::Error;
use teehistorian::Item;
use teehistorian::Reader;

fn process(path: &Path) -> Result<(), Error> {
    let mut buffer = Buffer::new();
    let mut reader = Reader::open(path, &mut buffer)?;
    let mut tick = None;
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
            _ => println!("{:8} {:?}", tick.expect("in tick"), item),
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
            println!("{}: {:?}", path.display(), err);
            process::exit(1);
        }
    }
}
