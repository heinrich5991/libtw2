use std::fmt;
use std::ops;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapacityError;

pub struct Buffer<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(buffer: &mut [u8]) -> Buffer {
        Buffer {
            buffer: buffer,
            len: 0,
        }
    }
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), CapacityError> {
        self.extend(bytes.iter().cloned())
    }
    pub fn extend<I>(&mut self, bytes: I) -> Result<(), CapacityError>
        where I: Iterator<Item=u8>
    {
        let mut buf_iter = (&mut self.buffer[self.len..]).into_iter();
        for b in bytes {
            *unwrap_or_return!(buf_iter.next(), Err(CapacityError)) = b;
            self.len += 1;
        }
        Ok(())
    }
    pub fn advance(&mut self, num_bytes: usize) {
        assert!(self.len + num_bytes <= self.buffer.len());
        self.len += num_bytes;
    }
    pub fn init(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
    pub fn uninit_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[self.len..]
    }
}

impl<'a> ops::Deref for Buffer<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
}

impl<'a> ops::DerefMut for Buffer<'a> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.len]
    }
}

impl<'a> fmt::Debug for Buffer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&**self).fmt(f)
    }
}
