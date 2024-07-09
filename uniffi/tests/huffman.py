import functools
import os.path
import unittest

import libtw2_huffman as huffman

@functools.cache
def test_cases():
    with open(os.path.join(os.path.dirname(__file__), "../../huffman/data/test_cases")) as f:
        test_cases = f.read()
    return [tuple(bytes.fromhex(part) for part in line.split("#")) for line in test_cases.splitlines()]

class Huffman(unittest.TestCase):
    def test_decompress(self):
        for uncompressed, compressed in test_cases():
            self.assertEqual(huffman.decompress(compressed), uncompressed)

    def test_compress(self):
        for uncompressed, compressed in test_cases():
            # The reference implementation sometimes adds an unnecessary null
            # byte at the end of the compression.
            our_compressed = huffman.compress(uncompressed)
            if len(our_compressed) + 1 == len(compressed):
                our_compressed += b"\x00"
            self.assertEqual(our_compressed, compressed)

if __name__ == '__main__':
    unittest.main()
