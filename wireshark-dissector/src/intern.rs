use std::sync::atomic;
use std::cmp;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::str;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Interned(NonNull<u8>);

#[derive(Default)]
pub struct Interner {
    lookup: HashMap<&'static str, Interned>,
    buffer: &'static mut [u8],
    last_buffer_len: usize,
}

fn intern_into_bytes(s: &str, bytes: &'static mut [u8]) -> Interned {
    assert!(s.len() + 1 == bytes.len());
    bytes[..s.len()].copy_from_slice(s.as_bytes());
    Interned(NonNull::new(bytes.as_mut_ptr()).expect("nonnull"))
}

impl Interner {
    pub fn new() -> Interner {
        Default::default()
    }
    fn next_buffer_len(&self) -> usize {
        if self.last_buffer_len == 0 {
            4096
        } else {
            self.last_buffer_len * 2
        }
    }
    fn intern_impl(&mut self, s: &str) -> Interned {
        for b in s.bytes() {
            if b == 0 {
                panic!("can't intern strings with embedded NULs: {:?}", s);
            }
        }
        let size = s.len() + 1;
        if size > self.buffer.len() {
            let next_buffer_len = self.next_buffer_len();
            if size > next_buffer_len {
                let zeros: Vec<u8> = (0..size).map(|_| 0).collect();
                return intern_into_bytes(s, Box::leak(zeros.into_boxed_slice()));
            }
            let zeros: Vec<u8> = (0..next_buffer_len).map(|_| 0).collect();
            self.buffer = Box::leak(zeros.into_boxed_slice());
        }
        let buffer = mem::replace(&mut self.buffer, &mut []);
        let (buffer, remaining) = buffer.split_at_mut(size);
        self.buffer = remaining;
        intern_into_bytes(s, buffer)
    }
    pub fn intern(&mut self, s: &str) -> Interned {
        if let Some(&i) = self.lookup.get(s) {
            return i;
        }
        let interned = self.intern_impl(s);
        assert!(self.lookup.insert(interned.as_str(), interned).is_none());
        interned
    }
    pub fn intern_static_with_nul(&mut self, s: &'static str) -> Interned {
        assert!(s.bytes().rev().next() == Some(0),
            "static strings for interning have to end in NUL");
        let s = &s[..s.len() - 1];
        if let Some(&i) = self.lookup.get(s) {
            return i;
        }
        for b in s.bytes() {
            if b == 0 {
                panic!("can't intern strings with embedded NULs: {:?}", s);
            }
        }
        let interned = Interned(
            NonNull::new(s.as_bytes().as_ptr() as *mut _)
                .expect("nonnull")
        );
        assert!(self.lookup.insert(s, interned).is_none());
        interned
    }
}

impl Interned {
    pub fn c(self) -> *const c_char {
        self.0.as_ptr() as *mut c_char as *const c_char
    }
    pub fn as_c_str(self) -> &'static CStr {
        unsafe {
            CStr::from_ptr(self.c())
        }
    }
    pub fn as_bytes(self) -> &'static [u8] {
        self.as_c_str().to_bytes()
    }
    pub fn as_bytes_with_nul(self) -> &'static [u8] {
        self.as_c_str().to_bytes_with_nul()
    }
    pub fn as_str(self) -> &'static str {
        unsafe {
            str::from_utf8_unchecked(self.as_bytes())
        }
    }
    pub fn as_str_with_nul(self) -> &'static str {
        unsafe {
            str::from_utf8_unchecked(self.as_bytes_with_nul())
        }
    }
}
impl PartialOrd for Interned {
    fn partial_cmp(&self, other: &Interned) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Interned {
    fn cmp(&self, other: &Interned) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl fmt::Debug for Interned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
impl fmt::Display for Interned {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

static mut INTERNER: Option<Interner> = None;
static INTERNER_IN_USE: atomic::AtomicBool = atomic::AtomicBool::new(false);
pub fn intern(s: &str) -> Interned {
    let result;
    assert!(!INTERNER_IN_USE.swap(true, atomic::Ordering::SeqCst));
    unsafe {
        result = INTERNER.get_or_insert(Interner::new()).intern(s);
    }
    INTERNER_IN_USE.store(false, atomic::Ordering::SeqCst);
    result
}
pub fn intern_static_with_nul(s: &'static str) -> Interned {
    let result;
    assert!(!INTERNER_IN_USE.swap(true, atomic::Ordering::SeqCst));
    unsafe {
        result = INTERNER.get_or_insert(Interner::new())
            .intern_static_with_nul(s);
    }
    INTERNER_IN_USE.store(false, atomic::Ordering::SeqCst);
    result
}
