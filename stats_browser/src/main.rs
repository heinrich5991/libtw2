#![cfg(not(test))]

extern crate clap;
extern crate logger;
extern crate stats_browser;

use clap::App;
use clap::Arg;
use stats_browser::StatsBrowser;
use stats_browser::StatsBrowserCb;
use stats_browser::tracker_fstd;
use stats_browser::tracker_json;

fn run_browser<T: StatsBrowserCb>(tracker: &mut T) {
    let mut browser = match StatsBrowser::new(tracker) {
        Some(b) => b,
        None => {
            panic!("Failed to bind socket.");
        },
    };
    browser.run();
}

fn main() {
    logger::init();

    let matches = App::new("stats_browser")
        .version("0.0.1")
        .author("heinrich5991 <heinrich5991@gmail.com>")
        .about("Tracks changes in the Teeworlds server list")
        .arg(Arg::with_name("format")
            .short("f")
            .long("format")
            .takes_value(true)
            .value_name("FORMAT")
            .default_value("fstd")
            .possible_value("fstd")
            .possible_value("json")
            .help("Output format")
        )
        .arg(Arg::with_name("filename")
            .long("filename")
            .takes_value(true)
            .value_name("FILENAME")
            .default_value("servers.json")
            .help("Output filename (only used for json tracker)")
        )
        .get_matches();

    match matches.value_of("format").unwrap() {
        "fstd" => {
            let mut tracker = tracker_fstd::Tracker::new();
            tracker.start();
            run_browser(&mut tracker);
        }
        "json" => {
            let filename = String::from(matches.value_of("filename").unwrap());
            let mut tracker = tracker_json::Tracker::new(filename);
            tracker.start();
            run_browser(&mut tracker);
        }
        _ => unreachable!(),
    }
}
