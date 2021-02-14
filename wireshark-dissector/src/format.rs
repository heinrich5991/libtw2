use arrayvec::Array;
use arrayvec::ArrayString;
use std::fmt;
use std::mem;

pub struct Bitfield<'a> {
    bytes: &'a [u8],
    mask: u64,
}

impl<'a> Bitfield<'a> {
    pub fn new(bytes: &[u8], mask: u64) -> Bitfield {
        assert!(bytes.len() <= mem::size_of_val(&mask));
        Bitfield {
            bytes: bytes,
            mask: mask,
        }
    }
}

impl<'a> fmt::Display for Bitfield<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result: ArrayString<[u8; 256]> = ArrayString::new();
        for (i, &b) in self.bytes.iter().enumerate() {
            if i != 0 {
                result.push_str(" ").unwrap();
            }
            let mask_shift = (self.bytes.len() - i - 1) * 8;
            for j in (0..8).rev() {
                let bit = 1 << j;
                let mask_bit = 1 << (j + mask_shift);
                result.push_str(match (mask_bit & self.mask != 0, b & bit != 0) {
                    (false, _) => ".",
                    (true, false) => "0",
                    (true, true) => "1",
                }).unwrap();
                if j == 4 {
                    result.push_str(" ").unwrap();
                }
            }
        }
        f.write_str(&result)
    }
}

pub struct CommaSeparated<A: Array<Item = u8>> {
    empty: bool,
    string: ArrayString<A>,
}

impl<A: Array<Item = u8>> CommaSeparated<A> {
    pub fn new() -> CommaSeparated<A> {
        CommaSeparated {
            empty: true,
            string: ArrayString::new(),
        }
    }
    pub fn add(&mut self, s: &str) {
        if !self.empty {
            self.string.push_str(", ").unwrap();
        }
        self.empty = false;
        self.string.push_str(s).unwrap();
    }
    pub fn or<'a>(&'a self, default: &'a str) -> &'a str {
        if !self.empty {
            &self.string
        } else {
            default
        }
    }
}

pub struct NumBytes {
    num: usize,
}

impl NumBytes {
    pub fn new(num: usize) -> NumBytes {
        NumBytes {
            num,
        }
    }
}

impl fmt::Display for NumBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.num != 1 {
            write!(f, "{} bytes", self.num)
        } else {
            write!(f, "{} byte", self.num)
        }
    }
}


#[cfg(test)]
mod test {
    use std::fmt;
    use super::Bitfield;

    fn assert_fmt<T: fmt::Display>(t: T, expected: &str) {
        assert_eq!(t.to_string(), expected);
    }

    #[test]
    fn format() {
        assert_fmt(Bitfield::new(&[], 0), "");
        assert_fmt(Bitfield::new(&[0], 0), ".... ....");
        assert_fmt(Bitfield::new(&[0], 0b1100_1100), "00.. 00..");
        assert_fmt(Bitfield::new(&[0b10101010; 2], 0b0011_0011_1010_1010),
            "..10 ..10 1.1. 1.1.");
    }
}
