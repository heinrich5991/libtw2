#![cfg(not(test))]

use clap::value_t;
use clap::App;
use clap::Arg;
use clap::Values;
use libtw2_stats_browser::tracker_fstd;
use libtw2_stats_browser::tracker_json;
use libtw2_stats_browser::StatsBrowser;
use libtw2_stats_browser::StatsBrowserCb;
use std::collections::HashSet;
use uuid::Uuid;

fn run_browser<T: StatsBrowserCb>(tracker: &mut T, masters: Vec<(String, bool)>) {
    let browser = if masters.is_empty() {
        StatsBrowser::new(tracker)
    } else {
        StatsBrowser::new_without_masters(tracker).map(|mut browser| {
            for (master, nobackcompat) in masters {
                browser.add_master(master, nobackcompat);
            }
            browser
        })
    };
    if let Some(mut browser) = browser {
        browser.run();
    } else {
        panic!("Failed to bind socket.");
    }
}

fn main() {
    libtw2_logger::init();

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
            .default_value("dump.json")
            .help("Output filename (only used for json tracker)")
        )
        .arg(Arg::with_name("locations")
            .long("locations")
            .takes_value(true)
            .value_name("LOCATIONS")
            .help("IP to continent locations database filename (only used for json tracker, libloc format, can be obtained from https://location.ipfire.org/databases/1/location.db.xz)")
        )
        .arg(Arg::with_name("seed")
            .long("seed")
            .takes_value(true)
            .value_name("SEED")
            .help("UUID seed to use for fake secrets of the reported servers (only used for json tracker, useful if you want to merge output of multiple stats_browser instances)")
        )
        .arg(Arg::with_name("master")
            .long("master")
            .takes_value(true)
            .value_name("MASTER")
            .multiple(true)
            .number_of_values(1)
            .help("Master server to use [default: master1.teeworlds.com to master4.teeworlds.com]")
        )
        .arg(Arg::with_name("master-nobackcompat")
            .long("master-nobackcompat")
            .takes_value(true)
            .value_name("MASTER")
            .multiple(true)
            .number_of_values(1)
            .help("Master server to use, has to support the NOBACKCOMPAT extension to not send servers obtained from the newer HTTPS masters")
        )
        .get_matches();

    fn add_masters(
        masters: &mut Vec<(String, bool)>,
        seen: &mut HashSet<String>,
        args: Option<Values<'_>>,
        nobackcompat: bool,
    ) {
        if let Some(args) = args {
            for arg in args {
                if !seen.insert(arg.to_owned()) {
                    panic!("master {:?} seen twice", arg);
                }
                masters.push((arg.to_owned(), nobackcompat));
            }
        }
    }
    let mut masters = Vec::new();
    {
        let mut seen = HashSet::new();
        add_masters(&mut masters, &mut seen, matches.values_of("master"), false);
        add_masters(
            &mut masters,
            &mut seen,
            matches.values_of("master-nobackcompat"),
            true,
        );
    }

    match matches.value_of("format").unwrap() {
        "fstd" => {
            let mut tracker = tracker_fstd::Tracker::new();
            tracker.start();
            run_browser(&mut tracker, masters);
        }
        "json" => {
            let filename = String::from(matches.value_of("filename").unwrap());
            let locations = matches.value_of("locations").map(String::from);
            let seed: Option<Uuid> = if matches.is_present("seed") {
                Some(value_t!(matches, "seed", Uuid).unwrap_or_else(|e| e.exit()))
            } else {
                None
            };
            let mut tracker = tracker_json::Tracker::new(filename, locations, seed);
            tracker.start();
            run_browser(&mut tracker, masters);
        }
        _ => unreachable!(),
    }
}
