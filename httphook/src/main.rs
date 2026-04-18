use std::thread;
use std::time::Duration;

fn main() {
    libtw2_httphook::register_server_6(8303);
    thread::sleep(Duration::from_millis(100));
}
