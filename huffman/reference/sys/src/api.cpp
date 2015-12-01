#include "api.h"

#include "teeworlds/huffman.h"

struct huffman
{
	mutable CHuffman Huffman;
};

extern "C" size_t huffman_size(void)
{
	return sizeof(struct huffman);
}

extern "C" void huffman_init(struct huffman *huffman, const unsigned frequencies[256])
{
	huffman->Huffman.Init(frequencies);
}

extern "C" int huffman_compress(const struct huffman *huffman, const void *input,
		int input_size, void *output, int output_size)
{
	return huffman->Huffman.Compress(input, input_size, output, output_size);
}

extern "C" int huffman_decompress(const struct huffman *huffman, const void *input,
		int input_size, void *output, int output_size)
{
	return huffman->Huffman.Decompress(input, input_size, output, output_size);
}
