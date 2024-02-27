use bencher::benchmark_group;
use bencher::benchmark_main;
use bencher::black_box;
use bencher::Bencher;
use common::num::Cast;
use huffman::Huffman;
use huffman_reference::Huffman as HuffmanReference;
use itertools::Itertools;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::u8;

fn read_file<T, F: FnMut(String) -> T>(filename: &str, f: F) -> Vec<T> {
    BufReader::new(File::open(filename).unwrap())
        .lines()
        .map(|ml| ml.unwrap())
        .map(f)
        .collect()
}

fn frequencies_default() -> Vec<u32> {
    read_file("data/frequencies", |l| u32::from_str_radix(&l, 10).unwrap())
}

fn huffman_default() -> Huffman {
    huffman::instances::TEEWORLDS
}

fn huffman_reference_default() -> HuffmanReference {
    HuffmanReference::from_frequencies(&frequencies_default())
}

fn test_cases() -> Vec<(Vec<u8>, Vec<u8>)> {
    read_file("data/test_cases", |l| {
        let mut v = l
            .split('#')
            .map(|hex_bytes| {
                hex_bytes
                    .split(' ')
                    .map(|hex| u8::from_str_radix(hex, 16).unwrap())
                    .collect()
            })
            .collect_vec()
            .into_iter();
        assert_eq!(v.len(), 2);
        let uncompressed = v.next().unwrap();
        let compressed = v.next().unwrap();
        (uncompressed, compressed)
    })
}

fn from_frequencies(b: &mut Bencher) {
    let frequencies = frequencies_default();
    b.iter(|| {
        black_box(Huffman::from_frequencies(&frequencies));
    });
}

fn compressed_len_bug(b: &mut Bencher) {
    let h = huffman_default();
    let test_cases = test_cases();
    b.iter(|| {
        for &(ref uncompressed, _) in &test_cases {
            black_box(h.compressed_len_bug(uncompressed));
        }
    });
    b.bytes = test_cases
        .iter()
        .map(|&(ref uncompressed, _)| uncompressed.len())
        .fold(0, |s, l| s + l.u64());
}

fn compressed_len(b: &mut Bencher) {
    let h = huffman_default();
    let test_cases = test_cases();
    b.iter(|| {
        for &(ref uncompressed, _) in &test_cases {
            black_box(h.compressed_len(uncompressed));
        }
    });
    b.bytes = test_cases
        .iter()
        .map(|&(ref uncompressed, _)| uncompressed.len())
        .fold(0, |s, l| s + l.u64());
}

fn compress(b: &mut Bencher) {
    let h = huffman_default();
    let test_cases = test_cases();
    let buffer = &mut (0..10240).map(|_| 0).collect_vec();
    b.iter(|| {
        for &(ref uncompressed, _) in &test_cases {
            let _ = black_box(h.compress(uncompressed, &mut buffer[..]));
        }
    });
    b.bytes = test_cases
        .iter()
        .map(|&(ref uncompressed, _)| uncompressed.len())
        .fold(0, |s, l| s + l.u64());
}

fn decompress(b: &mut Bencher) {
    let h = huffman_default();
    let test_cases = test_cases();
    let buffer = &mut (0..10240).map(|_| 0).collect_vec();
    b.iter(|| {
        for &(_, ref compressed) in &test_cases {
            let _ = black_box(h.decompress(compressed, &mut buffer[..]));
        }
    });
    b.bytes = test_cases
        .iter()
        .map(|&(_, ref compressed)| compressed.len())
        .fold(0, |s, l| s + l.u64());
}

fn from_frequencies_reference(b: &mut Bencher) {
    let frequencies = frequencies_default();
    b.iter(|| {
        black_box(HuffmanReference::from_frequencies(&frequencies));
    });
}

fn compress_reference(b: &mut Bencher) {
    let h = huffman_reference_default();
    let test_cases = test_cases();
    let buffer = &mut (0..10240).map(|_| 0).collect_vec();
    b.iter(|| {
        for &(ref uncompressed, _) in &test_cases {
            let _ = black_box(h.compress(uncompressed, &mut buffer[..]));
        }
    });
    b.bytes = test_cases
        .iter()
        .map(|&(ref uncompressed, _)| uncompressed.len())
        .fold(0, |s, l| s + l.u64());
}

fn decompress_reference(b: &mut Bencher) {
    let h = huffman_reference_default();
    let test_cases = test_cases();
    let buffer = &mut (0..10240).map(|_| 0).collect_vec();
    b.iter(|| {
        for &(_, ref compressed) in &test_cases {
            let _ = black_box(h.decompress(compressed, &mut buffer[..]));
        }
    });
    b.bytes = test_cases
        .iter()
        .map(|&(_, ref compressed)| compressed.len())
        .fold(0, |s, l| s + l.u64())
}

benchmark_group!(
    compression,
    from_frequencies,
    compressed_len_bug,
    compressed_len,
    compress,
    decompress,
    from_frequencies_reference,
    compress_reference,
    decompress_reference,
);
benchmark_main!(compression);
