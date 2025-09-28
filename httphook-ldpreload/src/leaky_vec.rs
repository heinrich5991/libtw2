use std::ops;
use std::ptr;
use std::sync::atomic;
use std::sync::atomic::AtomicPtr;
use std::sync::Mutex;

pub struct LeakyVec<T: Clone + 'static> {
    write: Mutex<Vec<T>>,
    read: AtomicPtr<&'static [T]>,
}

fn leak<T: Clone>(elements: &[T]) -> &'static mut &'static [T] {
    Box::leak(Box::new(Box::leak(elements.to_vec().into_boxed_slice())))
}

impl<T: Clone + 'static> LeakyVec<T> {
    pub const fn new() -> LeakyVec<T> {
        LeakyVec {
            write: Mutex::new(Vec::new()),
            read: AtomicPtr::new(ptr::null_mut()),
        }
    }
    pub fn push_and_commit(&self, element: T) {
        let mut write = self.write.lock().unwrap();
        write.push(element);
        self.read.store(leak(&write), atomic::Ordering::Release);
    }
    #[expect(dead_code)]
    pub fn push(&self, element: T) {
        let mut write = self.write.lock().unwrap();
        write.push(element);
    }
    #[expect(dead_code)]
    pub fn commit(&self) {
        let write = self.write.lock().unwrap();
        self.read.store(leak(&write), atomic::Ordering::Release);
    }
}

impl<T: Clone + 'static> ops::Deref for LeakyVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        let read = self.read.load(atomic::Ordering::Acquire);
        if read.is_null() {
            return &[];
        }
        unsafe { *read }
    }
}
