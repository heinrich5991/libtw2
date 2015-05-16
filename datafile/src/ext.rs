use std::io;

pub trait ReadComplete: io::Read {
    fn read_complete(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let read = try!(self.read(buf));
        if read != buf.len() {
            panic!("buffer not completely filled");
        }
        Ok(())
    }
}

impl<T:io::Read> ReadComplete for T { }
