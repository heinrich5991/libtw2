use hexdump::hexdump;
use msg::System;
use msg::system::Info;
use packer::with_packer;

#[test]
fn encode_info() {
    let mut buf = [0; 4096];
    let buf = &mut buf[..];
    let result = with_packer(buf, |p| System::Info(Info {
        version: b"abc",
        password: Some(b"ok"),
    }).encode(p)).unwrap();
    println!("");
    hexdump(result);
    assert!(result == b"\x03abc\0ok\0");
}
