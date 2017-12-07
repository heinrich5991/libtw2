pub trait Callback {
    type Error;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}

struct Reader {
    buffer: Vec<u8>,
}
