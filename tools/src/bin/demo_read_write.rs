extern crate demo;

use demo::{Reader, Writer};
use std::env;
use std::error::Error;
use std::fs;
use std::process;

fn next_arg(args: &mut env::Args, program_name: &str) -> String {
    match args.next() {
        None => {
            println!("USAGE: {} <DEMO> <OUT>", program_name);
            process::exit(-1);
        }
        Some(arg) => arg,
    }
}

fn main() {
    let mut args = env::args();
    let program_name = args.next().unwrap();
    let demo_path = next_arg(&mut args, &program_name);
    let out_path = next_arg(&mut args, &program_name);

    if let Err(err) = read_write(&demo_path, &out_path) {
        println!("Error: {}", err);
        process::exit(-1);
    }
}

fn read_write(input: &str, output: &str) -> Result<(), Box<dyn Error>> {
    let input_file = fs::File::open(input)?;
    let output_file = fs::File::create(output)?;
    let mut reader = Reader::new(input_file, &mut warn::Ignore)?;
    let mut writer = Writer::new(
        output_file,
        reader.net_version(),
        reader.map_name(),
        reader.map_sha256(),
        reader.map_crc(),
        reader.kind(),
        reader.length(),
        reader.timestamp(),
        reader.map_data(),
    )?;
    while let Some(chunk) = reader.read_chunk(&mut warn::Ignore)? {
        writer.write_chunk(chunk)?;
    }
    Ok(())
}
