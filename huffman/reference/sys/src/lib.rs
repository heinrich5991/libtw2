extern crate libc;

#[link(name = "huffman")]
extern "C" {
    pub fn huffman_size() -> libc::size_t;
    pub fn huffman_init(huffman: *mut libc::c_void, frequencies: *const [libc::c_uint; 256]);
    pub fn huffman_compress(
        huffman: *const libc::c_void,
        input: *const libc::c_void,
        input_size: libc::c_int,
        output: *mut libc::c_void,
        output_size: libc::c_int,
    ) -> libc::c_int;
    pub fn huffman_decompress(
        huffman: *const libc::c_void,
        input: *const libc::c_void,
        input_size: libc::c_int,
        output: *mut libc::c_void,
        output_size: libc::c_int,
    ) -> libc::c_int;
}
