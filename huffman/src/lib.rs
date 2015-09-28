#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)] extern crate quickcheck;

extern crate arrayvec;
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
    bits: u32,
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
