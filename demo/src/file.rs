use common::digest::Sha256;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use warn::Warn;

use crate::format;
use crate::format::Warning;
use crate::raw;
use crate::writer;

pub struct Reader {
    data: Box<dyn io::Read>,
    raw: raw::Reader,
}

impl Reader {
    pub fn new<W: Warn<Warning>>(warn: &mut W, file: File) -> Result<Reader, raw::Error> {
        let mut data = BufReader::new(file);
        let raw = raw::Reader::new(warn, &mut data)?;
        Ok(Reader {
            data: Box::new(data),
            raw: raw,
        })
    }
    pub fn open<W, P>(warn: &mut W, path: P) -> Result<Reader, raw::Error>
    where
        W: Warn<Warning>,
        P: AsRef<Path>,
    {
        Reader::new(warn, File::open(path)?)
    }
    pub fn version(&self) -> format::Version {
        self.raw.version()
    }
    pub fn net_version(&self) -> &[u8] {
        self.raw.net_version()
    }
    pub fn map_name(&self) -> &[u8] {
        self.raw.map_name()
    }
    pub fn map_size(&self) -> u32 {
        self.raw.map_size()
    }
    pub fn map_crc(&self) -> u32 {
        self.raw.map_crc()
    }
    pub fn timestamp(&self) -> &[u8] {
        self.raw.timestamp()
    }
    pub fn timeline_markers(&self) -> &[format::Tick] {
        self.raw.timeline_markers()
    }
    pub fn read_chunk<'a, W>(
        &'a mut self,
        warn: &mut W,
    ) -> Result<Option<format::Chunk<'a>>, raw::Error>
    where
        W: Warn<Warning>,
    {
        Ok(self.raw.read_chunk(warn, &mut self.data)?)
    }
}

pub struct Writer {
    file: BufWriter<File>,
    raw: writer::Writer,
}

impl Writer {
    fn new_impl(
        file: File,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Option<Sha256>,
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> io::Result<Writer> {
        let mut file = BufWriter::new(file);
        let raw = writer::Writer::new(
            &mut file,
            net_version,
            map_name,
            map_sha256,
            map_crc,
            type_,
            timestamp,
        )?;
        Ok(Writer {
            file: file,
            raw: raw,
        })
    }
    pub fn new(
        file: File,
        net_version: &[u8],
        map_name: &[u8],
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> io::Result<Writer> {
        Self::new_impl(file, net_version, map_name, None, map_crc, type_, timestamp)
    }
    pub fn new_ddnet(
        file: File,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Sha256,
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> io::Result<Writer> {
        Self::new_impl(
            file,
            net_version,
            map_name,
            Some(map_sha256),
            map_crc,
            type_,
            timestamp,
        )
    }
    pub fn create<P: AsRef<Path>>(
        path: P,
        net_version: &[u8],
        map_name: &[u8],
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> io::Result<Writer> {
        fn inner(
            path: &Path,
            net_version: &[u8],
            map_name: &[u8],
            map_crc: u32,
            type_: &[u8],
            timestamp: &[u8],
        ) -> io::Result<Writer> {
            Writer::new_impl(
                File::create(path)?,
                net_version,
                map_name,
                None,
                map_crc,
                type_,
                timestamp,
            )
        }
        inner(
            path.as_ref(),
            net_version,
            map_name,
            map_crc,
            type_,
            timestamp,
        )
    }
    pub fn create_ddnet<P: AsRef<Path>>(
        path: P,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Sha256,
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> io::Result<Writer> {
        fn inner(
            path: &Path,
            net_version: &[u8],
            map_name: &[u8],
            map_sha256: Sha256,
            map_crc: u32,
            type_: &[u8],
            timestamp: &[u8],
        ) -> io::Result<Writer> {
            Writer::new_impl(
                File::create(path)?,
                net_version,
                map_name,
                Some(map_sha256),
                map_crc,
                type_,
                timestamp,
            )
        }
        inner(
            path.as_ref(),
            net_version,
            map_name,
            map_sha256,
            map_crc,
            type_,
            timestamp,
        )
    }
    pub fn write_chunk(&mut self, chunk: format::Chunk) -> io::Result<()> {
        self.raw.write_chunk(&mut self.file, chunk)
    }
    pub fn write_tick(&mut self, keyframe: bool, tick: format::Tick) -> io::Result<()> {
        self.raw.write_tick(&mut self.file, keyframe, tick)
    }
    pub fn write_snapshot(&mut self, snapshot: &[u8]) -> io::Result<()> {
        self.raw.write_snapshot(&mut self.file, snapshot)
    }
    pub fn write_snapshot_delta(&mut self, delta: &[u8]) -> io::Result<()> {
        self.raw.write_snapshot_delta(&mut self.file, delta)
    }
    pub fn write_message(&mut self, msg: &[u8]) -> io::Result<()> {
        self.raw.write_message(&mut self.file, msg)
    }
}
