extern crate libc;
extern crate num;
extern crate huffman_reference_sys as sys;

use num::ToPrimitive;

pub struct Huffman {
    huffman: Vec<u8>,
}

impl Huffman {
    pub fn from_frequencies(frequencies: &[u32]) -> Result<Huffman,()> {
        assert!(frequencies.len() == 256);
        let array = unsafe { &*(frequencies as *const _ as *const _) };
        Huffman::from_frequencies_array(array)
    }
    pub fn from_frequencies_array(frequencies: &[u32; 256]) -> Result<Huffman,()> {
        let huffman_size = unsafe { sys::huffman_size() }.to_usize().unwrap();
        let huffman = Vec::with_capacity(huffman_size);
        let mut result = Huffman { huffman: huffman };
        // Implicit assumption that `c_uint == u32`. Screams when it breaks, so
        // it's fine.
        unsafe { sys::huffman_init(result.inner_huffman_mut(), frequencies); }
        Ok(result)
    }
    pub fn compress<'a>(&self, input: &[u8], buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        let result_len = unsafe {
            sys::huffman_compress(
                self.inner_huffman(),
                input.as_ptr() as *const _, input.len().to_i32().unwrap(),
                buffer.as_ptr() as *mut _, buffer.len().to_i32().unwrap()
            )
        };
        match result_len.to_usize() {
            Some(l) => Some(&buffer[..l]),
            None => None,
        }
    }
    pub fn decompress<'a>(&self, input: &[u8], buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        let result_len = unsafe {
            sys::huffman_decompress(
                self.inner_huffman(),
                input.as_ptr() as *const _, input.len().to_i32().unwrap(),
                buffer.as_ptr() as *mut _, buffer.len().to_i32().unwrap()
            )
        };
        match result_len.to_usize() {
            Some(l) => Some(&buffer[..l]),
            None => None,
        }
    }
    fn inner_huffman_mut(&mut self) -> *mut libc::c_void {
        self.huffman.as_mut_ptr() as *mut _
    }
    fn inner_huffman(&self) -> *const libc::c_void {
        self.huffman.as_ptr() as *const _
    }
}
