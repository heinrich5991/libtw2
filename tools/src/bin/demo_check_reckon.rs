use clap::App;
use clap::Arg;
use libtw2_demo::ddnet;
use libtw2_gamenet_ddnet::Protocol as DDNet;
use libtw2_warn as warn;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    libtw2_logger::init();
    let matches = App::new("Teehistorian reader")
        .about(
            "Reads teehistorian file and dumps its contents in a human-readable\
                text stream",
        )
        .arg(
            Arg::with_name("INPUT_DEMO")
                .help("Sets the demo file to read")
                .required(true),
        )
        .get_matches();

    let input = matches.value_of("INPUT_DEMO").unwrap();
    let input_file = BufReader::new(File::open(input)?);
    let mut reader = ddnet::DemoReader::<DDNet>::new(input_file, &mut warn::Log)?;
    let mut last_tick = None;
    let mut tee_ticks = HashMap::new();
    while let Some(chunk) = reader.next_chunk(&mut warn::Log)? {
        match chunk {
            ddnet::Chunk::Snapshot(snap) => {
                for (obj, id) in snap {
                    if let libtw2_gamenet_ddnet::SnapObj::Character(chr) = obj {
                        let tick = chr.character_core.tick;
                        assert!(tick <= last_tick.unwrap());
                        if let Some(last_tick) = tee_ticks.insert(*id, tick) {
                            // assert!(tick >= last_tick); // Breaks
                            if tick < last_tick {
                                println!(
                                    "Character {id} with decreased tick: {last_tick} -> {tick}"
                                );
                            }
                            if last_tick == 0 {
                                println!("Character {id} going from tick 0 -> {tick}");
                            }
                        } else {
                            println!("Character {id} starting at tick {tick}");
                        }
                    }
                }
            }
            ddnet::Chunk::Tick(t) => last_tick = Some(t),
            _ => {}
        }
    }
    Ok(())
}
