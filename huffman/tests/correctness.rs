extern crate huffman;
extern crate itertools;

use huffman::Huffman;
use itertools::Itertools;
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

#[test]
fn from_frequencies() {
    let input = read_file("data/input", |l| {
        u32::from_str_radix(&l, 10).unwrap()
    });
    let repr = read_file("data/repr", |l| l);

    let h = Huffman::from_frequencies(&input).unwrap();
    let generated_repr = h.repr().into_iter().map(|r| r.to_string()).collect_vec();

    assert_eq!(generated_repr, repr);
}
