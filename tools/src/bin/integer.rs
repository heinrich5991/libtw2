extern crate arrayvec;
extern crate clap;
extern crate hexdump;
extern crate packer;
extern crate tools;

use arrayvec::ArrayVec;
use clap::App;
use clap::Arg;
use packer::Unpacker;
use packer::with_packer;
use tools::unhexdump::Unhexdump;
use tools::warn_stdout::Stdout;

fn main() {
    let matches = App::new("Teeworlds variable-length integer encoding")
        .about("Takes an integer and writes out its Teeworlds variable-length\
                or vice versa")
        .arg(Arg::with_name("decode")
            .short("d")
            .long("decode")
            .help("Decode a hexdump instead, hexdump must be enclosed in pipes,\
                   sorry :(") // TODO
        )
        .arg(Arg::with_name("INTEGER")
            .help("The integer to de-/encode")
            .required(true)
        )
        .get_matches();
    let integer = matches.value_of("INTEGER").unwrap();
    if !matches.is_present("decode") {
        let integer: i32 = integer.parse().expect("invalid input");
        let mut out: ArrayVec<[u8; 32]> = ArrayVec::new();
        with_packer(&mut out, |mut p| p.write_int(integer)).unwrap();
        hexdump::hexdump(&out);
    } else {
        let mut un = Unhexdump::new();
        un.feed(integer.as_bytes()).unwrap();
        let encoded = un.into_inner().unwrap();
        let mut up = Unpacker::new(&encoded);
        println!("{}", up.read_int(&mut Stdout).unwrap());
        up.finish(&mut Stdout);
    }
}
