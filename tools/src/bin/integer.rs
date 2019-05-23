extern crate clap;
extern crate hexdump;
extern crate packer;

use clap::App;
use clap::Arg;
use packer::with_packer;

fn main() {
    let matches = App::new("Teeworlds variable-length integer encoding")
        .about("Takes an integer and writes out its Teeworlds variable-length encoding")
        .arg(Arg::with_name("INTEGER")
            .help("The integer to encode")
            .required(true)
        )
        .get_matches();
    let integer = matches.value_of("INTEGER").unwrap();
    let integer: i32 = integer.parse().expect("invalid input");
    let mut out = Vec::with_capacity(1024);
    with_packer(&mut out, |mut p| p.write_int(integer)).unwrap();
    hexdump::hexdump(&out);
}
