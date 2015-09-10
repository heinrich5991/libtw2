use arrayvec::ArrayVec;
use itertools::Itertools;
use std::fmt;
use std::io::Write;
use std::iter;
use std::ops;
use std::slice::Chunks;
use std::slice;
use std::str;

const SEGMENT_LENGTH: usize = 4;
// CHUNK_LENGTH should be a multiple of SEGMENT_LENGTH
const CHUNK_LENGTH: usize = 16;

const NUM_SEGMENTS_PER_CHUNK: usize = ((CHUNK_LENGTH + SEGMENT_LENGTH - 1) / SEGMENT_LENGTH);

const BUFFER_LENGTH: usize = 64;

// Must be UTF-8!
type BufferImpl = ArrayVec<[u8; BUFFER_LENGTH]>;

#[derive(Clone)]
pub struct Buffer {
    inner: BufferImpl,
}

impl Buffer {
    fn new(inner: BufferImpl) -> Buffer {
        Buffer { inner: inner }
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl ops::Deref for Buffer {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.inner) }
    }
}

pub struct Hexdump<'a> {
    len: usize,
    chunks: iter::Enumerate<slice::Chunks<'a, u8>>,
    summary_done: bool,
}

pub fn hexdump(bytes: &[u8]) {
    hexdump_iter(bytes).foreach(|s| println!("{}", s));
}

pub fn hexdump_iter(bytes: &[u8]) -> Hexdump {
    Hexdump::new(bytes)
}

impl<'a> Hexdump<'a> {
    fn new(bytes: &[u8]) -> Hexdump {
        Hexdump {
            len: bytes.len(),
            chunks: bytes.chunks(CHUNK_LENGTH).enumerate(),
            summary_done: false,
        }
    }
}

fn once<T,F:FnOnce()->T>(once: &mut bool, f: F) -> Option<T> {
    if !*once {
        *once = true;
        Some(f())
    } else {
        None
    }
}

impl<'a> Iterator for Hexdump<'a> {
    type Item = Buffer;
    fn next(&mut self) -> Option<Buffer> {
        let summary_done = &mut self.summary_done;
        let len = self.len;
        self.chunks.next().map(hexdump_chunk)
            .or_else(|| once(summary_done, || hexdump_summary(len)))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> DoubleEndedIterator for Hexdump<'a> {
    fn next_back(&mut self) -> Option<Buffer> {
        let chunks = &mut self.chunks;
        let len = self.len;
        once(&mut self.summary_done, || hexdump_summary(len))
            .or_else(|| chunks.next_back().map(hexdump_chunk))
    }
}

impl<'a> ExactSizeIterator for Hexdump<'a> {
    fn len(&self) -> usize {
        self.chunks.len() + if !self.summary_done { 1 } else { 0 }
    }
}

fn hexdump_summary(len: usize) -> Buffer {
    let mut buf: ArrayVec<[u8; BUFFER_LENGTH]> = ArrayVec::new();
    buf.write_all(b"    ").unwrap();
    for _ in 0..CHUNK_LENGTH {
        buf.write_all(b"   ").unwrap();
    }
    for _ in 1..NUM_SEGMENTS_PER_CHUNK {
        buf.write_all(b" ").unwrap();
    }
    write!(buf, "{:08x}", len).unwrap();

    Buffer::new(buf)
}

fn hexdump_chunk((i, chunk): (usize, &[u8])) -> Buffer {
    let offset = i * CHUNK_LENGTH;

    let mut buf: ArrayVec<[u8; BUFFER_LENGTH]> = ArrayVec::new();
    buf.write_all(b"|").unwrap();

    let mut first = true;
    let mut num_segments = 0;
    let mut num_bytes = 0;
    for segment in chunk.chunks(SEGMENT_LENGTH) {
        if first {
            first = false;
        } else {
            buf.write_all(b" ").unwrap();
        }

        num_bytes = 0;
        for &b in segment {
            write!(buf, "{:02x}", b).unwrap();
            num_bytes += 1;
        }
        num_segments += 1;
    }

    buf.write_all(b"| ").unwrap();
    for _ in num_bytes..SEGMENT_LENGTH {
        buf.write_all(b"  ").unwrap();
    }
    for _ in num_segments..NUM_SEGMENTS_PER_CHUNK {
        for _ in 0..SEGMENT_LENGTH {
            buf.write_all(b"  ").unwrap();
        }
        buf.write_all(b" ").unwrap();
    }

    for &b in chunk {
        if b < 0x20 || b >= 0x7f {
            buf.write_all(b".").unwrap();
        } else {
            write!(buf, "{}", b as char).unwrap();
        }
    }

    for _ in chunk.len()..CHUNK_LENGTH {
        buf.write_all(b" ").unwrap();
    }

    buf.write_all(b" ").unwrap();
    write!(buf, "{:08x}", offset).unwrap();

    Buffer::new(buf)
}

#[cfg(test)]
mod test {
    use super::hexdump_iter;

    use itertools::Itertools;
    use std::collections::HashSet;

    #[quickcheck]
    fn length(bytes: Vec<u8>) -> bool {
        let len = hexdump_iter(b"").next().unwrap().len();
        hexdump_iter(&bytes).all(|s| s.len() == len)
    }

    #[quickcheck]
    fn ascii_only_no_cc(bytes: Vec<u8>) -> bool {
        hexdump_iter(&bytes).all(|s| s.bytes().all(|b| 0x20 <= b && b < 0x7f))
    }

    #[quickcheck]
    fn summary(bytes: Vec<u8>) -> bool {
        usize::from_str_radix(hexdump_iter(&bytes).last().unwrap().trim(), 16).ok()
            == Some(bytes.len())
    }

    #[quickcheck]
    fn chars_existent(bytes: Vec<u8>) -> bool {
        let printable_chars: HashSet<_> = bytes.iter()
            .filter(|&&b| 0x20 <= b && b < 0x7f)
            .map(|&b| b as char)
            .collect();
        let lines = hexdump_iter(&bytes).map(|l| l.to_owned()).collect_vec();
        let printed_chars: HashSet<_> = lines.iter()
            .flat_map(|l| l.chars())
            .collect();

        printable_chars.is_subset(&printed_chars)
    }
}
