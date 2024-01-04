extern crate huffman;

use huffman::Huffman;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

fn read_file<T, F: FnMut(String) -> T>(filename: &str, f: F) -> Vec<T> {
    BufReader::new(File::open(filename).unwrap())
        .lines()
        .map(|ml| ml.unwrap())
        .map(f)
        .collect()
}

fn main() {
    let input = read_file("data/frequencies", |l| u32::from_str_radix(&l, 10).unwrap());

    for r in Huffman::from_frequencies(&input).repr() {
        println!("{}", r);
    }
}
