extern crate common;
extern crate huffman;
extern crate itertools;

use huffman::Huffman;
use itertools::Itertools;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::u8;

fn buffer() -> Vec<u8> {
    Vec::with_capacity(10240)
}

fn read_file<T, F: FnMut(String) -> T>(filename: &str, f: F) -> Vec<T> {
    BufReader::new(File::open(filename).unwrap())
        .lines()
        .map(|ml| ml.unwrap())
        .map(f)
        .collect()
}

fn huffman_default() -> Huffman {
    let frequencies = read_file("data/frequencies", |l| {
        u32::from_str_radix(&l, 10).unwrap()
    });

    Huffman::from_frequencies(&frequencies).unwrap()
}

fn test_cases() -> Vec<(Vec<u8>, Vec<u8>)> {
    read_file("data/test_cases", |l| {
        let mut v = l.split('#').map(|hex_bytes| {
            hex_bytes.split(' ').map(|hex| u8::from_str_radix(hex, 16).unwrap()).collect()
        }).collect_vec().into_iter();
        assert_eq!(v.len(), 2);
        let uncompressed = v.next().unwrap();
        let compressed = v.next().unwrap();
        (uncompressed, compressed)
    })
}

#[test]
fn from_frequencies() {
    let h = huffman_default();
    let repr = read_file("data/repr", |l| l);

    let generated_repr = h.repr().into_iter().map(|r| r.to_string()).collect_vec();

    assert_eq!(generated_repr, repr);
}

#[test]
fn compressed_len() {
    let h = huffman_default();

    for (uncompressed, compressed) in test_cases() {
        let compressed_len_bug = h.compressed_len_bug(&uncompressed);
        let compressed_len = h.compressed_len(&uncompressed);
        // The reference implementation occasionally adds an extra byte to the
        // compressed data.
        assert!(compressed_len <= compressed_len_bug);
        assert!(compressed_len_bug <= compressed_len + 1);

        assert_eq!(compressed_len_bug, compressed.len())
    }
}

#[test]
fn compress() {
    fn fuzzy_match(compressed: &[u8], potentially_buggy: &[u8]) {
        let mut potentially_buggy = potentially_buggy;
        if compressed.len() + 1 == potentially_buggy.len() {
            potentially_buggy = &potentially_buggy[..potentially_buggy.len() - 1];
        }
        assert_eq!(compressed, potentially_buggy);
    }

    let h = huffman_default();

    let mut buffer = buffer();
    for (uncompressed, compressed) in test_cases() {
        buffer.clear();
        fuzzy_match(h.compress_bug(&uncompressed, &mut buffer).unwrap(), &compressed);
    }
}

#[test]
fn compress_bug() {
    let h = huffman_default();

    let mut buffer = buffer();
    for (uncompressed, compressed) in test_cases() {
        buffer.clear();
        assert_eq!(h.compress_bug(&uncompressed, &mut buffer).unwrap(), &compressed[..]);
    }
}

#[test]
fn decompress() {
    let h = huffman_default();

    let mut buffer = buffer();
    for (uncompressed, compressed) in test_cases() {
        buffer.clear();
        assert_eq!(h.compress_bug(&uncompressed, &mut buffer).unwrap(), &compressed[..]);
    }
}

#[test]
fn decompress_extend_stream() {
    let h = huffman_default();

    let mut buffer = buffer();
    assert_eq!(&[0x00, 0x00, 0x00][..], h.decompress(&[0x57, 0xdc], &mut buffer).unwrap());
}
