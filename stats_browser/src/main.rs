#![cfg(not(test))]

extern crate env_logger;
extern crate stats_browser;

use stats_browser::StatsBrowser;
use stats_browser::tracker_fstd;

fn main() {
    env_logger::init().unwrap();

    let mut tracker = tracker_fstd::Tracker::new();
    tracker.start();
    let mut browser = match StatsBrowser::new(&mut tracker) {
        Some(b) => b,
        None => {
            panic!("Failed to bind socket.");
        },
    };
    browser.run();
}
