#ifndef HUFFMAN_API_H
#define HUFFMAN_API_H

#include <stddef.h>

struct huffman;

extern "C" size_t huffman_size(void);

extern "C" void huffman_init(struct huffman *huffman, const unsigned frequencies[256]);
extern "C" int huffman_compress(const struct huffman *huffman, const void *input,
		int input_size, void *output, int output_size);
extern "C" int huffman_decompress(const struct huffman *huffman, const void *input,
		int input_size, void *output, int output_size);

#endif
