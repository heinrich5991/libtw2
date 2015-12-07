#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)] extern crate quickcheck;

extern crate arrayvec;
#[macro_use] extern crate common;
extern crate itertools;
extern crate num;

use arrayvec::ArrayVec;
use itertools::Itertools;
use num::ToPrimitive;
use std::fmt::Write;
use std::fmt;
use std::slice;

const EOF: u16 = 256;
pub const NUM_SYMBOLS: u16 = EOF + 1;
const NUM_NODES: usize = NUM_SYMBOLS as usize * 2 - 1;
const ROOT_IDX: u16 = NUM_NODES as u16 - 1;
pub const NUM_FREQUENCIES: usize = 256;

pub struct Huffman {
    nodes: [Node; NUM_NODES],
}

pub struct Repr<'a> {
    repr: &'a [Node],
}

pub struct ReprIter<'a> {
    iter: slice::Iter<'a, Node>,
}

impl<'a> IntoIterator for Repr<'a> {
    type Item = SymbolRepr;
    type IntoIter = ReprIter<'a>;
    fn into_iter(self) -> ReprIter<'a> {
        ReprIter {
            iter: self.repr.iter(),
        }
    }
}

impl<'a> Iterator for ReprIter<'a> {
    type Item = SymbolRepr;
    fn next(&mut self) -> Option<SymbolRepr> {
        self.iter.next().map(|n| n.to_symbol_repr())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for ReprIter<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> DoubleEndedIterator for ReprIter<'a> {
    fn next_back(&mut self) -> Option<SymbolRepr> {
        self.iter.next_back().map(|n| n.to_symbol_repr())
    }
}

struct Bits {
    byte: u8,
    remaining_bits: u8,
}

impl Bits {
    fn new(byte: u8) -> Bits {
        Bits {
            byte: byte,
            remaining_bits: 8,
        }
    }
}

impl Iterator for Bits {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        if self.remaining_bits == 0 {
            return None;
        }
        self.remaining_bits -= 1;
        let result = (self.byte & 1) != 0;
        self.byte >>= 1;
        Some(result)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for Bits {
    fn len(&self) -> usize {
        self.remaining_bits.to_usize().unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
struct Frequency {
    frequency: u32,
    node_idx: u16,
}

impl Huffman {
    pub fn from_frequencies(frequencies: &[u32]) -> Result<Huffman,()> {
        assert!(frequencies.len() == 256);
        let array = unsafe { &*(frequencies as *const _ as *const _) };
        Huffman::from_frequencies_array(array)
    }
    pub fn from_frequencies_array(frequencies: &[u32; 256]) -> Result<Huffman,()> {
        let mut frequencies: ArrayVec<[_; 512]> = frequencies.iter()
            .cloned().enumerate().map(|(i, f)| {
                Frequency { frequency: f, node_idx: i.to_u16().unwrap() }
            }).collect();
        assert!(frequencies.push(Frequency { frequency: 1, node_idx: EOF }).is_none());

        let mut nodes: ArrayVec<[_; 1024]> =
            (0..NUM_SYMBOLS).map(|_| NODE_SENTINEL).collect();

        while frequencies.len() > 1 {
            // Sort in reverse (upper to lower)!
            frequencies.sort_by(|a, b| b.frequency.cmp(&a.frequency));

            // `frequencies.len() > 1`, so these always succeed.
            let freq1 = frequencies.pop().unwrap();
            let freq2 = frequencies.pop().unwrap();

            // Combine the nodes into one.
            let node = Node { children: [freq1.node_idx, freq2.node_idx] };
            let node_idx = nodes.len().to_u16().unwrap();
            let node_freq = Frequency {
                frequency: freq1.frequency.saturating_add(freq2.frequency),
                node_idx: node_idx,
            };

            assert!(nodes.push(node).is_none());
            assert!(frequencies.push(node_freq).is_none());
        }

        // We use a `top` variable as virtual extension of `stack` in order to
        // have less `unwrap`s.
        let mut stack: ArrayVec<[u16; 24]> = ArrayVec::new();
        let mut top = ROOT_IDX;

        let mut bits = 0;
        let mut first = true;

        // Use a depth-first traversal of the tree, exploring the left children
        // of each node first, in order to set the bit patterns of the leaves.
        loop {
            // On first iteration, don't try to go up the tree.
            if !first {
                if let Some(t) = stack.pop() {
                    top = t;
                } else {
                    break;
                }
                let b = 1 << stack.len().to_u8().unwrap();
                if bits & b != 0 {
                    bits &= !b;
                    continue;
                }
                bits |= b;
                assert!(stack.push(top).is_none());
                top = nodes[top.to_usize().unwrap()].children[1];
            }
            first = false;

            while top >= NUM_SYMBOLS {
                assert!(stack.push(top).is_none());
                top = nodes[top.to_usize().unwrap()].children[0];
            }

            nodes[top.to_usize().unwrap()] = SymbolRepr {
                bits: bits,
                num_bits: stack.len().to_u8().unwrap(),
            }.to_node();
        }

        let mut result = Huffman { nodes: [NODE_SENTINEL; NUM_NODES] };
        assert!(result.nodes.iter_mut().set_from(nodes.iter().cloned()) == NUM_NODES);
        Ok(result)
    }
    fn compressed_bit_len(&self, input: &[u8]) -> usize {
        input.iter().map(|&b| self.symbol_bit_length(b.to_u16().unwrap()))
         .fold(0, |s, a| s + a.to_usize().unwrap())
            + self.symbol_bit_length(EOF).to_usize().unwrap()
    }
    pub fn compressed_len(&self, input: &[u8]) -> usize {
        (self.compressed_bit_len(input) + 7) / 8
    }
    /// This function returns the number of bytes the reference implementation
    /// uses to compress the input bytes.
    ///
    /// This might differ by 1 from `compressed_len` in case the compressed bit
    /// stream would perfectly fit into bytes.
    pub fn compressed_len_bug(&self, input: &[u8]) -> usize {
        self.compressed_bit_len(input) / 8 + 1
    }
    pub fn compress<'a>(&self, input: &[u8], buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        /*
        let mut len = 0;
        {
            let mut output = buffer.into_iter();
            let mut output_byte = 0;
            let mut num_output_bits = 0;
            for &byte in input {
                let symbol = self.get_node(byte.to_u16().unwrap()).unwrap_err();
                let mut bits_written = 0;
                while symbol_bits > 0 {
                    output_byte |= (symbol_bits >> bits_written) << num_output_bits;
                    bits_written += 8 - num_output_bits;
                }
            }
        }
        */
        let _ = (input, buffer);
        unimplemented!();
    }
    pub fn decompress<'a>(&self, input: &[u8], buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        let mut len = 0;
        {
            let mut input = input.into_iter();
            let mut output = buffer.into_iter();
            let root = self.get_node(ROOT_IDX).unwrap();
            let mut node = root;
            'outer: loop {
                let &byte = input.next().unwrap_or(&0);
                for bit in Bits::new(byte) {
                    let new_idx = node.children[bit as usize];
                    if let Ok(n) = self.get_node(new_idx) {
                        node = n;
                    } else {
                        if new_idx == EOF {
                            break 'outer;
                        }
                        *unwrap_or_return!(output.next(), None) = new_idx.to_u8().unwrap();
                        len += 1;
                        node = root;
                    }
                }
            }
        }
        Some(&buffer[..len])
    }
    fn symbol_bit_length(&self, idx: u16) -> u32 {
        self.get_node(idx).unwrap_err().num_bits()
    }
    fn get_node(&self, idx: u16) -> Result<Node, SymbolRepr> {
        let n = self.nodes[idx.to_usize().unwrap()];
        if idx >= NUM_SYMBOLS {
            Ok(n)
        } else {
            Err(n.to_symbol_repr())
        }
    }
    pub fn repr(&self) -> Repr {
        Repr { repr: &self.nodes[..NUM_SYMBOLS.to_usize().unwrap()] }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Node {
    children: [u16; 2],
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SymbolRepr {
    bits: u32, // u24
    num_bits: u8,
}

const NODE_SENTINEL: Node = Node { children: [!0, !0] };

impl Node {
    fn to_symbol_repr(self) -> SymbolRepr {
        SymbolRepr {
            bits: ((self.children[0] & 0xff) as u32) << 16 | self.children[1] as u32,
            num_bits: (self.children[0] >> 8) as u8,
        }
    }
}

impl SymbolRepr {
    fn to_node(self) -> Node {
        assert!(self.bits >> 24 == 0);
        Node { children: [
            (self.num_bits as u16) << 8 | (self.bits >> 16) as u16,
            self.bits as u16
        ] }
    }
    pub fn num_bits(self) -> u32 {
        self.num_bits.to_u32().unwrap()
    }
    pub fn bit(self, idx: u32) -> bool {
        assert!(idx < self.num_bits());
        ((self.bits >> idx) & 1) != 0
    }
}

impl fmt::Debug for SymbolRepr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.num_bits() {
            try!(f.write_char(if self.bit(i) { '1' } else { '0' }));
        }
        Ok(())
    }
}

impl fmt::Display for SymbolRepr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod test {
    use super::Node;
    use super::SymbolRepr;

    #[quickcheck]
    fn roundtrip_node((v0, v1): (u16, u16)) -> bool {
        let n = Node { children: [v0, v1] };
        n.to_symbol_repr().to_node() == n
    }

    #[quickcheck]
    fn roundtrip_symbol((v0, v1): (u32, u8)) -> bool {
        let v0 = v0 ^ ((v0 >> 24) << 24);
        let s = SymbolRepr { bits: v0, num_bits: v1 };
        s.to_node().to_symbol_repr() == s
    }
}
