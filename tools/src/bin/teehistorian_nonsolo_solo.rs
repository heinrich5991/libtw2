use itertools::sorted;
use libtw2_teehistorian::format;
use libtw2_teehistorian::Buffer;
use libtw2_teehistorian::Reader;
use serde_derive::Serialize;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use uuid::Uuid;
use walkdir::WalkDir;

#[allow(dead_code)] // We add fields just for their `Debug` implementation.
#[derive(Debug)]
enum Error {
    Csv(csv::Error),
    Io(io::Error),
    Teehistorian(format::Error),
    WalkDir(walkdir::Error),
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Error {
        Error::Csv(e)
    }
}

impl From<libtw2_teehistorian::Error> for Error {
    fn from(e: libtw2_teehistorian::Error) -> Error {
        use libtw2_teehistorian::Error::*;
        match e {
            Teehistorian(i) => Error::Teehistorian(i),
            Io(i) => Error::Io(i),
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}

#[derive(Debug, Serialize)]
struct Record<'a> {
    path: &'a Path,
    game_uuid: Uuid,
    map_name: Cow<'a, str>,
    reset_file: Cow<'a, str>,
}

struct Config {
    ignore_ext: bool,
}

fn handle_dir<'a>(
    writer: &mut csv::Writer<Box<dyn Write>>,
    dir: &Path,
    config: &Config,
) -> Result<(), ()> {
    fn helper<'a>(
        writer: &mut csv::Writer<Box<dyn Write>>,
        dir: &Path,
        config: &Config,
    ) -> Result<(), Error> {
        let mut buffer = Buffer::new();
        for entry in WalkDir::new(dir).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
            let entry = entry?;
            if !config.ignore_ext && entry.path().extension() != Some(OsStr::new("teehistorian")) {
                continue;
            }
            if entry.file_type().is_dir() {
                continue;
            }
            buffer.clear();
            match Reader::open(entry.path(), &mut buffer) {
                Ok((mut header, _)) => {
                    writer.serialize(Record {
                        path: entry.path(),
                        game_uuid: header.game_uuid,
                        map_name: header.map_name,
                        reset_file: header
                            .config
                            .remove("sv_reset_file")
                            .unwrap_or(Default::default()),
                    })?;
                }
                Err(e) => {
                    eprintln!("{}: {:?}", entry.path().display(), e);
                }
            }
        }
        Ok(())
    }
    helper(writer, dir, config).map_err(|e| eprintln!("{}: {:?}", dir.display(), e))
}

fn handle_args(dirs: Vec<PathBuf>, config: &Config) -> Result<(), ()> {
    let mut csv_out: csv::Writer<Box<dyn Write>> = csv::Writer::from_writer(Box::new(io::stdout()));

    for dir in dirs {
        handle_dir(&mut csv_out, &dir, config)?;
    }

    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    libtw2_logger::init();

    let matches = App::new("Teehistorian indexer")
        .about(
            "Checks folders of teehistorian files for solo maps with \
                non-solo flexreset files",
        )
        .arg(
            Arg::with_name("DIRECTORY")
                .help("Directories to scan (current directory if none are given)")
                .multiple(true),
        )
        .arg(Arg::with_name("ignore-ext").long("--ignore-ext").help(
            "Don't check for the .teehistorian file extension before \
                   checking a file",
        ))
        .get_matches();

    let paths = matches.values_of_os("DIRECTORY");
    let config = Config {
        ignore_ext: matches.is_present("ignore-ext"),
    };

    let dirs = if let Some(p) = paths {
        sorted(p.into_iter().map(|p| PathBuf::from(p)))
    } else {
        vec![PathBuf::from(".")]
    };

    if handle_args(dirs, &config).is_err() {
        process::exit(1);
    }
}
