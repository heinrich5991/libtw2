use std::mem;
use std::raw;

pub fn relative_size_of_mult<T,U>(mult: usize) -> usize {
    assert!(mult * mem::size_of::<T>() % mem::size_of::<U>() == 0);
    mult * mem::size_of::<T>() / mem::size_of::<U>()
}

pub fn relative_size_of<T,U>() -> usize {
    relative_size_of_mult::<T,U>(1)
}

pub unsafe fn transmute<'a,T,U>(x: &'a [T]) -> &'a [U] {
    assert!(mem::min_align_of::<T>() % mem::min_align_of::<U>() == 0);
    mem::transmute(raw::Slice {
        data: x.as_ptr(),
        len: relative_size_of_mult::<T,U>(x.len()),
    })
}

pub unsafe fn transmute_mut<'a,T,U>(x: &'a mut [T]) -> &'a mut [U] {
    assert!(mem::min_align_of::<T>() % mem::min_align_of::<U>() == 0);
    mem::transmute(raw::Slice {
        data: x.as_ptr(),
        len: relative_size_of_mult::<T,U>(x.len()),
    })
}
