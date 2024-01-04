use buffer::with_buffer;
use buffer::Buffer;
use buffer::BufferRef;

use raw::Callback;

pub trait CallbackExt: Callback {
    fn read_buffer<'d, B: Buffer<'d>>(&mut self, buf: B) -> Result<Option<&'d [u8]>, Self::Error> {
        with_buffer(buf, |buf| self.read_buffer_ref(buf))
    }
    fn read_buffer_ref<'d, 's>(
        &mut self,
        mut buf: BufferRef<'d, 's>,
    ) -> Result<Option<&'d [u8]>, Self::Error> {
        unsafe {
            let read = unwrap_or_return!(self.read_at_most(buf.uninitialized_mut())?, Ok(None));
            buf.advance(read);
            Ok(Some(buf.initialized()))
        }
    }
}

impl<CB: Callback> CallbackExt for CB {}
