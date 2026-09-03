use clap::App;
use clap::Arg;
use libtw2_demo::ddnet;
use libtw2_demo::ddnet::DemoReader;
use libtw2_demo::ChunkType;
use libtw2_gamenet_ddnet::Protocol as DDNet;
use libtw2_warn as warn;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

trait SeekableDemo {
    fn stream_position(&mut self) -> Result<u64, Box<dyn Error>>;
    fn seek(&mut self, pos: u64) -> Result<(), Box<dyn Error>>;
    fn next_keyframe(&mut self) -> Result<Option<(i32, u64)>, Box<dyn Error>>;
    fn seek_keyframe_for_tick(
        &mut self,
        target_tick: i32,
    ) -> Result<Option<(i32, u64)>, Box<dyn Error>>;
}

impl SeekableDemo for DemoReader<'static, DDNet> {
    fn stream_position(&mut self) -> Result<u64, Box<dyn Error>> {
        self.stream_position().map_err(Into::into)
    }
    fn seek(&mut self, pos: u64) -> Result<(), Box<dyn Error>> {
        self.seek(pos).map_err(Into::into)
    }
    fn next_keyframe(&mut self) -> Result<Option<(i32, u64)>, Box<dyn Error>> {
        loop {
            let position = self.stream_position()?;
            let chunk_type = self.next_chunk_type()?;
            let Some(chunk) = self.next_chunk(&mut warn::Ignore)? else {
                assert!(chunk_type.is_none());
                return Ok(None);
            };
            assert!(matches!(
                (chunk_type, &chunk),
                (Some(ChunkType::Message), ddnet::Chunk::Message(_))
                    | (Some(ChunkType::Message), ddnet::Chunk::Invalid)
                    | (Some(ChunkType::Snapshot), ddnet::Chunk::Snapshot(_))
                    | (Some(ChunkType::Tick), ddnet::Chunk::Tick { .. })
                    | (None, ddnet::Chunk::Invalid)
            ));
            if let ddnet::Chunk::Tick {
                keyframe: true,
                tick,
            } = chunk
            {
                return Ok(Some((tick, position)));
            }
        }
    }
    fn seek_keyframe_for_tick(
        &mut self,
        target_tick: i32,
    ) -> Result<Option<(i32, u64)>, Box<dyn Error>> {
        self.next_keyframe_position_for_tick(target_tick, &mut warn::Ignore)
            .map_err(Into::into)
    }
}
impl SeekableDemo for libtw2_demo::Reader<'static> {
    fn stream_position(&mut self) -> Result<u64, Box<dyn Error>> {
        self.stream_position().map_err(Into::into)
    }
    fn seek(&mut self, pos: u64) -> Result<(), Box<dyn Error>> {
        self.seek(pos).map_err(Into::into)
    }
    fn next_keyframe(&mut self) -> Result<Option<(i32, u64)>, Box<dyn Error>> {
        loop {
            let position = self.stream_position()?;
            let Some(chunk) = self.read_chunk(&mut warn::Ignore)? else {
                return Ok(None);
            };
            if let libtw2_demo::RawChunk::Tick {
                keyframe: true,
                tick,
            } = chunk
            {
                return Ok(Some((tick, position)));
            }
        }
    }
    fn seek_keyframe_for_tick(
        &mut self,
        target_tick: i32,
    ) -> Result<Option<(i32, u64)>, Box<dyn Error>> {
        self.next_keyframe_position_for_tick(target_tick, &mut warn::Ignore)
            .map_err(Into::into)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    libtw2_logger::init();
    let matches = App::new("Demo Seek Tests")
        .about("Performs various seeks patterns on a demo to verify that seeking works as expected")
        .arg(
            Arg::with_name("INPUT_DEMO")
                .help("Sets the demo file to read")
                .required(true),
        )
        .arg(
            Arg::with_name("DDNET")
                .long("ddnet")
                .help("Interpret the demo as a DDNet demo"),
        )
        .get_matches();

    let input = matches.value_of("INPUT_DEMO").unwrap();
    let input_file = BufReader::new(File::open(input)?);
    let mut reader: Box<dyn SeekableDemo> = if matches.is_present("DDNET") {
        Box::new(DemoReader::<DDNet>::new(input_file, &mut warn::Log)?)
    } else {
        Box::new(libtw2_demo::Reader::new(input_file, &mut warn::Ignore)?)
    };
    let start = reader.stream_position()?;
    let mut keyframes = Vec::new();
    while let Some(keyframe) = reader.next_keyframe()? {
        keyframes.push(keyframe);
    }
    println!(
        "Demo parsed without errors, found {} keyframes",
        keyframes.len()
    );

    let mut tester = Tester {
        reader,
        start,
        keyframes,
    };
    tester.test_seek_all_keyframes_directly()?;
    println!("Seeking keyframes directly succeeded");
    Ok(())
}

struct Tester {
    reader: Box<dyn SeekableDemo>,
    start: u64,
    keyframes: Vec<(i32, u64)>,
}

impl Tester {
    fn test_seek_all_keyframes_directly(&mut self) -> Result<(), Box<dyn Error>> {
        // First seek from start
        self.reader.seek(self.start)?;
        for (tick, pos) in &self.keyframes {
            let found = self.reader.seek_keyframe_for_tick(*tick)?;
            assert_eq!(found, Some((*tick, *pos)));
        }
        // Seek from all previous keyframes
        for (i, (tick, pos)) in self.keyframes.iter().enumerate() {
            for (_, seek_pos) in &self.keyframes[..=i] {
                self.reader.seek(*seek_pos)?;
                let found = self.reader.seek_keyframe_for_tick(*tick)?;
                assert_eq!(found, Some((*tick, *pos)));
            }
        }
        // Seek to all previous keyframes
        for (i, (_, seek_pos)) in self.keyframes.iter().enumerate() {
            self.reader.seek(*seek_pos)?;
            for (tick, _) in &self.keyframes[..i] {
                let found = self.reader.seek_keyframe_for_tick(*tick)?;
                assert_eq!(found, None);
            }
        }
        Ok(())
    }
}
