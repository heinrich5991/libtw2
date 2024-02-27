fn main() {
    cc::Build::new()
        .include("src/include")
        .cpp(true)
        .cpp_link_stdlib(None)
        .file("src/teeworlds/huffman.cpp")
        .file("src/api.cpp")
        .compile("huffman");
}
