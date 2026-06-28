extern crate libtw2_snapshot_reference_sys as sys;

use libtw2_buffer::CapacityError;
use libtw2_common::num::Cast as _;
use std::convert::Infallible;

pub struct RawBuilder {
    builder: Vec<u8>,
    serialized_snap: Vec<i32>,
    serialize_ok: bool,
}

impl Default for RawBuilder {
    fn default() -> RawBuilder {
        let builder_size = unsafe { sys::snapshotbuilder_size() };
        let mut result = RawBuilder {
            builder: (0..builder_size).map(|_| 0).collect(),
            serialized_snap: Vec::with_capacity(16384),
            serialize_ok: false,
        };
        unsafe {
            sys::snapshotbuilder_init(result.inner_builder_mut());
        }
        result
    }
}

impl RawBuilder {
    fn inner_builder_mut(&mut self) -> *mut libc::c_void {
        self.builder.as_mut_ptr().cast::<libc::c_void>()
    }
    pub fn new() -> RawBuilder {
        RawBuilder::default()
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
    pub fn finish(mut self) -> RawSnap {
        const LEN: usize = 16384;
        assert!(self.serialized_snap.capacity() >= LEN);
        let buffer = self.serialized_snap.as_mut_ptr().cast::<[i32; LEN]>();
        let written = unsafe { sys::snapshotbuilder_finish(self.inner_builder_mut(), buffer) };
        self.serialize_ok = usize::try_from(written)
            // TODO (MSRV 1.76): Use `.inspect()`
            .map(|written| unsafe { self.serialized_snap.set_len(written) })
            .is_ok();
        unsafe {
            sys::snapshotbuilder_init(self.inner_builder_mut());
        }
        RawSnap(self)
    }
}

pub struct RawSnap(RawBuilder);

impl RawSnap {
    pub fn write_to_ints<'a>(&mut self, result: &'a mut [i32]) -> Result<&'a [i32], CapacityError> {
        if !self.0.serialize_ok {
            return Err(CapacityError);
        }
        if result.len() < self.0.serialized_snap.len() {
            return Err(CapacityError);
        }
        result[..self.0.serialized_snap.len()].copy_from_slice(&self.0.serialized_snap);
        Ok(&result[..self.0.serialized_snap.len()])
    }
    pub fn recycle(self) -> RawBuilder {
        self.0
    }
}

pub struct Delta {
    builder: Vec<u8>,
    prev_obj_size: Option<fn(u16) -> Option<u32>>,
}

impl Default for Delta {
    fn default() -> Delta {
        let delta_size = unsafe { sys::snapshotdelta_size() };
        let mut result = Delta {
            builder: (0..delta_size).map(|_| 0).collect(),
            prev_obj_size: None,
        };
        unsafe {
            sys::snapshotdelta_init(result.inner_delta_mut());
        }
        result
    }
}

impl Delta {
    fn inner_delta_mut(&mut self) -> *mut libc::c_void {
        self.builder.as_mut_ptr().cast::<libc::c_void>()
    }
    pub fn new() -> Delta {
        Delta::default()
    }
    #[allow(unpredictable_function_pointer_comparisons)] // only used for caching
    fn handle_obj_size(&mut self, obj_size: fn(u16) -> Option<u32>) {
        if self.prev_obj_size != Some(obj_size) {
            self.prev_obj_size = Some(obj_size);
            unsafe {
                sys::snapshotdelta_init(self.inner_delta_mut());
            }
            for type_ in 0..32768 {
                if let Some(size) = obj_size(type_) {
                    unsafe {
                        sys::snapshotdelta_set_static_size(
                            self.inner_delta_mut(),
                            type_,
                            size.usize(),
                        );
                    }
                }
            }
        }
    }
    pub fn create_raw_and_write_to_ints<'a>(
        &mut self,
        from: &RawSnap,
        to: &RawSnap,
        obj_size: fn(u16) -> Option<u32>,
        mut result: &'a mut [i32],
    ) -> Result<&'a [i32], CapacityError> {
        self.handle_obj_size(obj_size);
        if !from.0.serialize_ok || !to.0.serialize_ok {
            return Err(CapacityError);
        }
        if result.len() > 16384 {
            result = &mut result[..16384];
        }
        let result: &mut [i32; 16384] = result.try_into().map_err(|_| CapacityError)?;
        let written = unsafe {
            sys::snapshotdelta_create(
                self.inner_delta_mut(),
                from.0.serialized_snap.as_ptr(),
                to.0.serialized_snap.as_ptr(),
                result,
            )
        };
        match usize::try_from(written) {
            Ok(written) => Ok(&result[..written]),
            Err(_) => Err(CapacityError),
        }
    }
}
