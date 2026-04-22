extern crate libtw2_snapshot_reference_sys as sys;

use libtw2_buffer::CapacityError;
use std::convert::Infallible;

pub struct RawBuilder {
    builder: Vec<u8>,
}

impl Default for RawBuilder {
    fn default() -> RawBuilder {
        let builder_size = unsafe { sys::snapshotbuilder_size() };
        let mut result = RawBuilder {
            builder: (0..builder_size).map(|_| 0).collect(),
        };
        unsafe {
            sys::snapshotbuilder_init(result.inner_builder_mut());
        }
        result
    }
}

impl RawBuilder {
    fn inner_builder_mut(&mut self) -> *mut libc::c_void {
        self.builder.as_mut_ptr() as *mut _
    }
    pub fn new() -> RawBuilder {
        Default::default()
    }
    pub fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), Infallible> {
        unsafe {
            sys::snapshotbuilder_add_item(
                self.inner_builder_mut(),
                type_id,
                id,
                data.as_ptr(),
                data.len(),
            );
        }
        Ok(())
    }
    pub fn finish(self) -> RawSnap {
        RawSnap(self)
    }
}

pub struct RawSnap(RawBuilder);

impl RawSnap {
    pub fn write_to_ints<'a>(
        &mut self,
        _buf: &mut Vec<i32>,
        result: &'a mut [i32],
    ) -> Result<&'a [i32], CapacityError> {
        let result: &mut [i32; 16384] = result
            .try_into()
            .expect("need at least array of size 16384");
        let written = unsafe { sys::snapshotbuilder_finish(self.0.inner_builder_mut(), result) };
        match usize::try_from(written) {
            Ok(written) => Ok(&result[..written]),
            Err(_) => Err(CapacityError),
        }
    }
    pub fn recycle(mut self) -> RawBuilder {
        unsafe {
            sys::snapshotbuilder_init(self.0.inner_builder_mut());
        }
        self.0
    }
}
