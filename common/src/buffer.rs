use arrayvec::ArrayVec;
use arrayvec;
use std::fmt;
use std::ops;
use std::slice;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapacityError;

pub trait Buffer {
    fn write(&mut self, bytes: &[u8]) -> Result<(), CapacityError> {
        self.extend(bytes.iter().cloned())
    }
    fn extend<I>(&mut self, bytes: I) -> Result<(), CapacityError>
        where I: Iterator<Item=u8>;
    unsafe fn advance(&mut self, num_bytes: usize);
    unsafe fn uninit_mut(&mut self) -> &mut [u8];
    fn init(&self) -> &[u8];
    fn remaining(&self) -> usize;
}

pub struct SliceBuffer<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> SliceBuffer<'a> {
    pub fn new(buffer: &mut [u8]) -> SliceBuffer {
        SliceBuffer {
            buffer: buffer,
            len: 0,
        }
    }
    pub fn reset(&mut self) {
        self.len = 0;
    }
}

impl<'a> Buffer for SliceBuffer<'a> {
    fn extend<I>(&mut self, bytes: I) -> Result<(), CapacityError>
        where I: Iterator<Item=u8>
    {
        let mut buf_iter = (&mut self.buffer[self.len..]).into_iter();
        for b in bytes {
            *unwrap_or_return!(buf_iter.next(), Err(CapacityError)) = b;
            self.len += 1;
        }
        Ok(())
    }
    unsafe fn advance(&mut self, num_bytes: usize) {
        assert!(self.len + num_bytes <= self.buffer.len());
        self.len += num_bytes;
    }
    unsafe fn uninit_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[self.len..]
    }
    fn init(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
    fn remaining(&self) -> usize {
        self.buffer.len() - self.len
    }
}

impl<A: arrayvec::Array<Item=u8>> Buffer for ArrayVec<A> {
    fn extend<I>(&mut self, bytes: I) -> Result<(), CapacityError>
        where I: Iterator<Item=u8>
    {
        let mut bytes = bytes;
        Extend::extend(self, &mut bytes);
        if bytes.next().is_some() {
            return Err(CapacityError);
        }
        Ok(())
    }
    unsafe fn uninit_mut(&mut self) -> &mut [u8] {
        let capacity = self.capacity();
        let len = self.len();
        let remaining = capacity - len;
        slice::from_raw_parts_mut(self.as_mut_ptr().offset(len as isize), remaining)
    }
    unsafe fn advance(&mut self, num_bytes: usize) {
        let len = self.len();
        assert!(len + num_bytes <= self.capacity());
        self.set_len(len + num_bytes);
    }
    fn init(&self) -> &[u8] {
        self
    }
    fn remaining(&self) -> usize {
        self.len()
    }
}

impl<'a> ops::Deref for SliceBuffer<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
}

impl<'a> ops::DerefMut for SliceBuffer<'a> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..self.len]
    }
}

impl<'a> fmt::Debug for SliceBuffer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&**self).fmt(f)
    }
}
