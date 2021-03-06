#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate arrayvec;
extern crate buffer;
extern crate common;
extern crate itertools;

use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use common::num::Cast;
use itertools::Itertools;
use std::fmt::Write;
use std::fmt;
use std::slice;

pub mod instances;

const EOF: u16 = 256;
pub const NUM_SYMBOLS: u16 = EOF + 1;
const NUM_NODES: usize = NUM_SYMBOLS as usize * 2 - 1;
const ROOT_IDX: u16 = NUM_NODES as u16 - 1;
pub const NUM_FREQUENCIES: usize = 256;

pub struct Huffman {
    nodes: [Node; NUM_NODES],
}

#[derive(Debug)]
pub struct Error {
    _unused: (),
}

#[derive(Debug)]
pub enum DecompressionError {
    Capacity(buffer::CapacityError),
    InvalidInput,
}

#[derive(Clone)]
pub struct Repr<'a> {
    repr: &'a [Node],
}

#[derive(Clone)]
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
        self.remaining_bits.usize()
    }
}

#[derive(Clone, Copy, Debug)]
struct Frequency {
    frequency: u32,
    node_idx: u16,
}

impl Huffman {
    pub fn from_frequencies(frequencies: &[u32]) -> Result<Huffman, Error> {
        assert!(frequencies.len() == 256);
        let array = unsafe { &*(frequencies as *const _ as *const _) };
        Huffman::from_frequencies_array(array)
    }
    pub fn from_frequencies_array(frequencies: &[u32; 256]) -> Result<Huffman, Error> {
        let mut frequencies: ArrayVec<[_; 512]> = frequencies.iter()
            .cloned().enumerate().map(|(i, f)| {
                Frequency { frequency: f, node_idx: i.assert_u16() }
            }).collect();
        frequencies.push(Frequency { frequency: 1, node_idx: EOF });

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
            let node_idx = nodes.len().assert_u16();
            let node_freq = Frequency {
                frequency: freq1.frequency.saturating_add(freq2.frequency),
                node_idx: node_idx,
            };

            nodes.push(node);
            frequencies.push(node_freq);
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
                let b = 1 << stack.len().assert_u8();
                if bits & b != 0 {
                    bits &= !b;
                    continue;
                }
                bits |= b;
                stack.push(top);
                top = nodes[top.usize()].children[1];
            }
            first = false;

            while top >= NUM_SYMBOLS {
                stack.push(top);
                top = nodes[top.usize()].children[0];
            }

            nodes[top.usize()] = SymbolRepr {
                bits: bits,
                num_bits: stack.len().assert_u8(),
            }.to_node();
        }

        let mut result = Huffman { nodes: [NODE_SENTINEL; NUM_NODES] };
        assert!(result.nodes.iter_mut().set_from(nodes.iter().cloned()) == NUM_NODES);
        Ok(result)
    }
    fn compressed_bit_len(&self, input: &[u8]) -> usize {
        input.iter().map(|&b| self.symbol_bit_length(b.u16()))
         .fold(0, |s, a| s + a.usize())
            + self.symbol_bit_length(EOF).usize()
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
    pub fn compress<'a, B: Buffer<'a>>(&self, input: &[u8], buffer: B)
        -> Result<&'a [u8], buffer::CapacityError>
    {
        with_buffer(buffer, |b| self.compress_impl(input, b, false))
    }
    pub fn compress_bug<'a, B: Buffer<'a>>(&self, input: &[u8], buffer: B)
        -> Result<&'a [u8], buffer::CapacityError>
    {
        with_buffer(buffer, |b| self.compress_impl(input, b, true))
    }
    fn compress_impl<'d, 's>(&self, input: &[u8], mut buffer: BufferRef<'d, 's>, bug: bool)
        -> Result<&'d [u8], buffer::CapacityError>
    {
        unsafe {
            let len = self.compress_impl_unsafe(input, buffer.uninitialized_mut(), bug)
                           .map_err(|()| buffer::CapacityError)?;
            buffer.advance(len);
            Ok(buffer.initialized())
        }
    }
    fn compress_impl_unsafe(&self, input: &[u8], buffer: &mut [u8], bug: bool)
        -> Result<usize, ()>
    {
        let mut len = 0;
        let mut output = buffer.into_iter();
        let mut output_byte = 0;
        let mut num_output_bits = 0;
        for s in input.into_iter().map(|b| b.u16()).chain(Some(EOF)) {
            let symbol = self.get_node(s).unwrap_err();
            let mut bits_written = 0;
            if symbol.num_bits >= 8 - num_output_bits {
                output_byte |= (symbol.bits << num_output_bits) as u8;
                *output.next().ok_or(())? = output_byte;
                len += 1;
                bits_written += 8 - num_output_bits;
                while symbol.num_bits - bits_written >= 8 {
                    output_byte = (symbol.bits >> bits_written) as u8;
                    *output.next().ok_or(())? = output_byte;
                    len += 1;
                    bits_written += 8;
                }
                num_output_bits = 0;
                output_byte = 0;
            }
            output_byte |= ((symbol.bits >> bits_written) << num_output_bits) as u8;
            num_output_bits += symbol.num_bits - bits_written;
        }
        if num_output_bits > 0 || bug {
            *output.next().ok_or(())? = output_byte;
            len += 1;
        }
        Ok(len)
    }

    pub fn decompress<'a, B: Buffer<'a>>(&self, input: &[u8], buffer: B)
        -> Result<&'a [u8], DecompressionError>
    {
        with_buffer(buffer, |b| self.decompress_impl(input, b))
    }
    fn decompress_impl<'d, 's>(&self, input: &[u8], mut buffer: BufferRef<'d, 's>)
        -> Result<&'d [u8], DecompressionError>
    {
        unsafe {
            let len = self.decompress_unsafe(input, buffer.uninitialized_mut())
                           .map_err(|()| DecompressionError::Capacity(buffer::CapacityError))?;
            buffer.advance(len);
            Ok(buffer.initialized())
        }
    }
    fn decompress_unsafe(&self, input: &[u8], buffer: &mut [u8])
        -> Result<usize, ()>
    {
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
                        *output.next().ok_or(())? = new_idx.assert_u8();
                        len += 1;
                        node = root;
                    }
                }
            }
        }
        Ok(len)
    }
    fn symbol_bit_length(&self, idx: u16) -> u32 {
        self.get_node(idx).unwrap_err().num_bits()
    }
    fn get_node(&self, idx: u16) -> Result<Node, SymbolRepr> {
        let n = self.nodes[idx.usize()];
        if idx >= NUM_SYMBOLS {
            Ok(n)
        } else {
            Err(n.to_symbol_repr())
        }
    }
    pub fn repr(&self) -> Repr {
        Repr { repr: &self.nodes[..NUM_SYMBOLS.usize()] }
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
        self.num_bits.u32()
    }
    pub fn bit(self, idx: u32) -> bool {
        assert!(idx < self.num_bits());
        ((self.bits >> idx) & 1) != 0
    }
}

impl fmt::Debug for SymbolRepr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.num_bits() {
            f.write_char(if self.bit(i) { '1' } else { '0' })?;
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

    quickcheck! {
        fn roundtrip_node(v: (u16, u16)) -> bool {
            let n = Node { children: [v.0, v.1] };
            n.to_symbol_repr().to_node() == n
        }

        fn roundtrip_symbol(v: (u32, u8)) -> bool {
            let v0 = v.0 ^ ((v.0 >> 24) << 24);
            let s = SymbolRepr { bits: v0, num_bits: v.1 };
            s.to_node().to_symbol_repr() == s
        }
    }
}
