#![cfg(not(test))]

extern crate logger;
extern crate stats_browser;

use stats_browser::StatsBrowser;
use stats_browser::tracker_fstd;

fn main() {
    logger::init();

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
