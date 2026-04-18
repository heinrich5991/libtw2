# libtw2_huffman

Python bindings for `libtw2-huffman` which implements a homebrew compression
format using huffman coding that is used in Teeworlds/DDNet networking and demo
file format.

## Building from source

```bash
python -m pip install --upgrade build
python -m build
python -m pip install dist/libtw2_huffman-*.tar.gz
```

## Install from Pypi

```
pip install libtw2_huffman
```

## Sample usage

```python
import libtw2_huffman

assert libtw2_huffman.decompress(b'\xae\x95\x13\x5c\x09\x57\xc2\x16\x29\x6e\x00') == b'hello'
assert libtw2_huffman.compress(b'hello') == b'\xae\x95\x13\x5c\x09\x57\xc2\x16\x29\x6e\x00'
```

