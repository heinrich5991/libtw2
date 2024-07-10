use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

fn process(path: &Path) -> Result<(), Box<dyn Error>> {
    let _spec: libtw2_gamenet_spec::Spec = serde_json::from_slice(&fs::read(path)?)?;
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    libtw2_logger::init();

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
