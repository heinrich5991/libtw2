use std::mem;
use std::slice;

pub fn ref_slice<T>(r: &T) -> &[T] {
    unsafe { slice::from_raw_parts(r, 1) }
}

pub fn mut_ref_slice<T>(r: &mut T) -> &mut [T] {
    unsafe { slice::from_raw_parts_mut(r, 1) }
}

pub fn relative_size_of_mult<T,U>(mult: usize) -> usize {
    assert!(mult * mem::size_of::<T>() % mem::size_of::<U>() == 0);
    mult * mem::size_of::<T>() / mem::size_of::<U>()
}

pub fn relative_size_of<T,U>() -> usize {
    relative_size_of_mult::<T,U>(1)
}

pub unsafe fn transmute<T,U>(x: &[T]) -> &[U] {
    assert!(mem::min_align_of::<T>() % mem::min_align_of::<U>() == 0);
    slice::from_raw_parts(x.as_ptr() as *const U, relative_size_of_mult::<T,U>(x.len()))
}

pub unsafe fn transmute_mut<T,U>(x: &mut [T]) -> &mut [U] {
    assert!(mem::min_align_of::<T>() % mem::min_align_of::<U>() == 0);
    slice::from_raw_parts_mut(x.as_ptr() as *mut U, relative_size_of_mult::<T,U>(x.len()))
}
