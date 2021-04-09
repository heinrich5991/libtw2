extern crate cc;

use std::env;
use std::path::PathBuf;

fn non_windows_msvc() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-undefined,dynamic_lookup");
}

fn windows_msvc() {
    let top_level = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR")
        .expect("missing CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR")
        .expect("missing OUT_DIR"));
    let target = env::var_os("TARGET").expect("missing TARGET")
        .into_string().expect("invalid TARGET");

    let lib = cc::windows_registry::find_tool(&target, "lib.exe").expect("link.exe not found");
    let def_file = top_level.join("sys\\src\\libwireshark.def");
    let def_file = def_file.into_os_string().into_string().expect("invalid CARGO_MANIFEST_DIR");

    let lib_file = out_dir.join("libwireshark.lib");
    let lib_file = lib_file.into_os_string().into_string().expect("invalid OUT_DIR");

    let result = lib.to_command()
        .arg(format!("/def:{}", def_file))
        .arg(format!("/out:{}", lib_file))
        .status()
        .expect("failed to execute lib.exe");
    assert!(result.success());

    println!("cargo:rustc-cdylib-link-arg={}", lib_file);
}

fn main() {
    let msvc = env::var_os("CARGO_CFG_TARGET_VENDOR").map(|v| v == "pc").unwrap_or(false) &&
        env::var_os("CARGO_CFG_TARGET_FAMILY").map(|v| v == "windows").unwrap_or(false) &&
        env::var_os("CARGO_CFG_TARGET_ENV").map(|v| v == "msvc").unwrap_or(false);
    if msvc {
        windows_msvc();
    } else {
        non_windows_msvc();
    }
}
