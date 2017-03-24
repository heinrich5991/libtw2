extern crate demo;
extern crate logger;
extern crate warn;

use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use warn::Warn;

#[derive(Default)]
struct ErrorStats {
    demo_warnings: HashMap<demo::Warning, u64>,
    demo_errors: HashMap<demo::format::Error, u64>,
    io_errors: Vec<io::Error>,
    ok: u64,
}

fn update_error_stats(stats: &mut ErrorStats, err: demo::Error) {
    match err {
        demo::Error::Demo(e) => *stats.demo_errors.entry(e).or_insert(0) += 1,
        demo::Error::Io(e) => stats.io_errors.push(e),
    }
}

fn print_error_stats(error_stats: &ErrorStats) {
    for (e, c) in &error_stats.demo_errors {
        println!("{:?}: {}", e, c);
    }
    for (w, c) in &error_stats.demo_warnings {
        println!("{:?}: {}", w, c);
    }
    for e in &error_stats.io_errors {
        println!("{:?}", e);
    }
    println!("ok: {}", error_stats.ok);
}

fn process<W: Warn<demo::Warning>>(warn: &mut W, path: &Path)
    -> Result<(), demo::Error>
{
    let reader = demo::Reader::open(warn, path)?;
    println!("{}", path.display());
    println!("version: {:?}", reader.version());
    println!("net_version: {}", String::from_utf8_lossy(reader.net_version()));
    println!("map_name: {}", String::from_utf8_lossy(reader.map_name()));
    println!("map_size: {}", reader.map_size());
    println!("map_crc: {:x}", reader.map_crc());
    println!("timestamp: {}", String::from_utf8_lossy(reader.timestamp()));
    println!();
    Ok(())
}

fn main() {
    logger::init();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut error_stats = ErrorStats::default();
    for arg in args {
        have_args = true;
        let path = Path::new(&arg);
        match process(warn::closure(&mut |w| {
            *error_stats.demo_warnings.entry(w).or_insert(0) += 1;
            println!("{}: {:?}", path.display(), w);
        }), path) {
            Ok(()) => error_stats.ok += 1,
            Err(err) => {
                println!("{}: {:?}", path.display(), err);
                update_error_stats(&mut error_stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <DEMO>...", program_name.to_string_lossy());
        return;
    }
    print_error_stats(&error_stats);
}
