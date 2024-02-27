use datafile as df;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io;
use std::path::Path;

fn entity_name(index: u8) -> Option<&'static str> {
    Some(match index {
        0x00 => "None",
        0x01 => "Coll",
        0x02 => "Deat",
        0x03 => "Unho",
        0x3c => "Stop",
        0x3d => "StpS",
        0x3e => "StpA",
        0xc0 => "SpBr",
        0xc1 => "SpRe",
        0xc2 => "SpBl",
        0xc3 => "FlRe",
        0xc4 => "FlBl",
        0xc5 => "Shie",
        0xc6 => "Hear",
        0xc7 => "Shot",
        0xc8 => "Gren",
        0xc9 => "Ninj",
        0xca => "Lase",
        _ => return None,
    })
}

#[derive(Clone, Copy)]
pub struct Entity(pub u8);

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Entity(inner) = *self;
        match entity_name(inner) {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "0x{:02x}", inner),
        }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Default)]
struct ErrorStats {
    map_errors: HashMap<map::format::Error, u64>,
    df_errors: HashMap<df::format::Error, u64>,
    io_errors: Vec<io::Error>,
    ok: u64,
}

fn update_error_stats(stats: &mut ErrorStats, err: map::Error) {
    match err {
        map::Error::Map(e) => {
            *stats.map_errors.entry(e).or_insert(0) += 1;
        }
        map::Error::Df(df::Error::Df(e)) => {
            *stats.df_errors.entry(e).or_insert(0) += 1;
        }
        map::Error::Df(df::Error::Io(e)) => {
            stats.io_errors.push(e);
        }
    }
}

fn print_error_stats(error_stats: &ErrorStats) {
    for (e, c) in &error_stats.map_errors {
        println!("{:?}: {}", e, c);
    }
    for (e, c) in &error_stats.df_errors {
        println!("{:?}: {}", e, c);
    }
    for e in &error_stats.io_errors {
        println!("{:?}", e);
    }
    println!("ok: {}", error_stats.ok);
}

fn process<D, P>(path: &Path, process_inner: P, stats: &mut D) -> Result<(), map::Error>
where
    P: FnOnce(&Path, df::Reader, &mut D) -> Result<(), map::Error>,
{
    let reader = df::Reader::open(path)?;
    process_inner(path, reader, stats)
}

pub fn stats<D, P, S>(mut process_inner: P, summary: S)
where
    D: Default,
    P: FnMut(&Path, df::Reader, &mut D) -> Result<(), map::Error>,
    S: FnOnce(&D),
{
    logger::init();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut error_stats = ErrorStats::default();
    let mut stats = D::default();
    for arg in args {
        have_args = true;
        match process(Path::new(&arg), &mut process_inner, &mut stats) {
            Ok(()) => error_stats.ok += 1,
            Err(err) => {
                println!("{}: {:?}", arg.to_string_lossy(), err);
                update_error_stats(&mut error_stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <MAP>...", program_name.to_string_lossy());
        return;
    }
    print_error_stats(&error_stats);
    println!("--------");
    summary(&stats);
}
