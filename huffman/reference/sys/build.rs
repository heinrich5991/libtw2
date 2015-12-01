extern crate gcc;

fn main() {
    let mut cfg = gcc::Config::new();
    cfg.include("src/include");
    cfg.cpp(true);
    cfg.cpp_link_stdlib(None);
    cfg.file("src/teeworlds/huffman.cpp");
    cfg.file("src/api.cpp");
    cfg.compile("libhuffman.a");
}
