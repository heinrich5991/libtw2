extern crate clap;
extern crate gamenet_spec;
extern crate logger;
extern crate serde_json;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process;

fn process(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let _spec: gamenet_spec::Spec = serde_json::from_reader(file)?;
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Gamenet spec reader")
        .about("Reads a gamenet spec file and does nothing with it.")
        .arg(
            Arg::with_name("SPEC")
                .help("Sets the gamenet spec file to read")
                .required(true),
        )
        .get_matches();

    let path = Path::new(matches.value_of_os("SPEC").unwrap());

    match process(path) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}: {:?}", path.display(), err);
            process::exit(1);
        }
    }
}
