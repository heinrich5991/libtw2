use std::env;

fn non_windows_msvc() {
    println!("cargo:rustc-cdylib-link-arg=-Wl,-undefined,dynamic_lookup");
}

fn main() {
    let msvc = env::var_os("CARGO_CFG_TARGET_VENDOR")
        .map(|v| v == "pc")
        .unwrap_or(false)
        && env::var_os("CARGO_CFG_TARGET_FAMILY")
            .map(|v| v == "windows")
            .unwrap_or(false)
        && env::var_os("CARGO_CFG_TARGET_ENV")
            .map(|v| v == "msvc")
            .unwrap_or(false);
    if !msvc {
        non_windows_msvc();
    }
}
