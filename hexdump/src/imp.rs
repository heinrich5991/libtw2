use arrayvec::ArrayString;
use itertools::Itertools;
use std::fmt;
use std::fmt::Write;
use std::iter;
use std::ops;
use std::slice;

const SEGMENT_LENGTH: usize = 4;
// CHUNK_LENGTH should be a multiple of SEGMENT_LENGTH
const CHUNK_LENGTH: usize = 16;

const NUM_SEGMENTS_PER_CHUNK: usize = ((CHUNK_LENGTH + SEGMENT_LENGTH - 1) / SEGMENT_LENGTH);

const BUFFER_LENGTH: usize = 64;

type BufferImpl = ArrayString<[u8; BUFFER_LENGTH]>;

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
        &self.inner
    }
}

pub struct Hexdump<'a> {
    len: usize,
    chunks: iter::Enumerate<slice::Chunks<'a, u8>>,
    summary_done: bool,
}

pub fn sanitize_byte(byte: u8) -> char {
    if 0x20 <= byte && byte < 0x7f {
        byte as char
    } else {
        '.'
    }
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
    let mut buf = BufferImpl::new();
    buf.write_str("    ").unwrap();
    for _ in 0..CHUNK_LENGTH {
        buf.write_str("   ").unwrap();
    }
    for _ in 1..NUM_SEGMENTS_PER_CHUNK {
        buf.write_str(" ").unwrap();
    }
    write!(buf, "{:08x}", len).unwrap();

    Buffer::new(buf)
}

fn hexdump_chunk((i, chunk): (usize, &[u8])) -> Buffer {
    let offset = i * CHUNK_LENGTH;

    let mut buf = BufferImpl::new();
    buf.write_str("|").unwrap();

    let mut first = true;
    let mut num_segments = 0;
    let mut num_bytes = 0;
    for segment in chunk.chunks(SEGMENT_LENGTH) {
        if first {
            first = false;
        } else {
            buf.write_str(" ").unwrap();
        }

        num_bytes = 0;
        for &b in segment {
            write!(buf, "{:02x}", b).unwrap();
            num_bytes += 1;
        }
        num_segments += 1;
    }

    buf.write_str("| ").unwrap();
    for _ in num_bytes..SEGMENT_LENGTH {
        buf.write_str("  ").unwrap();
    }
    for _ in num_segments..NUM_SEGMENTS_PER_CHUNK {
        for _ in 0..SEGMENT_LENGTH {
            buf.write_str("  ").unwrap();
        }
        buf.write_str(" ").unwrap();
    }

    for &b in chunk {
        write!(buf, "{}", sanitize_byte(b)).unwrap();
    }

    for _ in chunk.len()..CHUNK_LENGTH {
        buf.write_str(" ").unwrap();
    }

    buf.write_str(" ").unwrap();
    write!(buf, "{:08x}", offset).unwrap();

    Buffer::new(buf)
}

#[cfg(test)]
mod test {
    use super::CHUNK_LENGTH;
    use super::hexdump_iter;
    use super::sanitize_byte;

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

    #[quickcheck]
    fn line_count(bytes: Vec<u8>) -> bool {
        let expected = (bytes.len() + CHUNK_LENGTH - 1) / CHUNK_LENGTH + 1;
        hexdump_iter(&bytes).len() == expected
            && hexdump_iter(&bytes).count() == expected
    }

    #[test]
    fn test_sanitize_byte() {
        use num::ToPrimitive;
        for i in 0..256u16 {
            let i = i.to_u8().unwrap();
            assert!(sanitize_byte(i) == '.' || sanitize_byte(i) == i as char);
        }
    }
}
