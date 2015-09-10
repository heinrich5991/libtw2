extern crate arrayvec;
extern crate void;

use arrayvec::ArrayVec;
use std::io::Write;
use std::str;
use void::ResultVoidExt;

const SEGMENT_LENGTH: usize = 4;
// CHUNK_LENGTH should be a multiple of SEGMENT_LENGTH
const CHUNK_LENGTH: usize = 16;

const NUM_SEGMENTS_PER_CHUNK: usize = ((CHUNK_LENGTH + SEGMENT_LENGTH - 1) / SEGMENT_LENGTH);

const BUFFER_LENGTH: usize = 64;

pub fn hexdump(bytes: &[u8]) {
    hexdump_raw(|s| Ok(println!("{}", s)), bytes).void_unwrap();
}

pub fn hexdump_raw<E,F:FnMut(&str)->Result<(),E>>(f: F, bytes: &[u8]) -> Result<(),E> {
    let mut f = f;
    for (i, chunk) in bytes.chunks(CHUNK_LENGTH).enumerate() {
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
        write!(buf, "{:08x}", i * CHUNK_LENGTH).unwrap();
        try!(f(unsafe { str::from_utf8_unchecked(&buf) }));
    }
    {
        let mut buf: ArrayVec<[u8; BUFFER_LENGTH]> = ArrayVec::new();
        buf.write_all(b"    ").unwrap();
        for _ in 0..CHUNK_LENGTH {
            buf.write_all(b"   ").unwrap();
        }
        for _ in 1..NUM_SEGMENTS_PER_CHUNK {
            buf.write_all(b" ").unwrap();
        }
        write!(buf, "{:08x}", bytes.len()).unwrap();
        try!(f(unsafe { str::from_utf8_unchecked(&buf) }));
    }
    Ok(())
}
