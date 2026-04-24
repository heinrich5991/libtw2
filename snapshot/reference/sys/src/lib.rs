#[link(name = "snapshot")]
extern "C" {
    pub fn snapshotbuilder_size() -> usize;
    pub fn snapshotbuilder_init(snapshotbuilder: *mut libc::c_void);
    pub fn snapshotbuilder_add_item(
        snapshotbuilder: *mut libc::c_void,
        type_: u16,
        id: u16,
        data: *const i32,
        data_len: usize,
    );
    pub fn snapshotbuilder_finish(
        snapshotbuilder: *mut libc::c_void,
        buffer: *mut [i32; 16384],
    ) -> isize;

    pub fn snapshotdelta_size() -> usize;
    pub fn snapshotdelta_init(snapshotdelta: *mut libc::c_void);
    pub fn snapshotdelta_set_static_size(snapshotdelta: *mut libc::c_void, type_: u16, len: usize);
    pub fn snapshotdelta_create(
        snapshotdelta: *mut libc::c_void,
        from: *const i32,
        to: *const i32,
        delta: *mut [i32; 16384],
    ) -> isize;
    pub fn snapshotdelta_unpack(
        snapshotdelta: *mut libc::c_void,
        from: *const i32,
        to: *mut [i32; 16384],
        delta: *const i32,
        delta_len: usize,
    ) -> isize;
}
