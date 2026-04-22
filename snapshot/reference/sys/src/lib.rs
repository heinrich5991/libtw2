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
}
