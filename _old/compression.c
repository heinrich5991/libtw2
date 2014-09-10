#include "compression.h"

#include "error.h"

#include <zlib.h>

uint32_t tw_comp_crc(uint32_t crc, void *buf, size_t length)
{
	return crc32(crc, buf, length);
}

int tw_comp_uncompress(void *dst, size_t *dst_size, const void *src, size_t src_size)
{
	int zlib_err = uncompress(dst, dst_size, src, src_size);
	if(zlib_err != Z_OK)
	{
		tw_error_set(TW_ERRNO_COMP_ERROR, "zlib error during uncompression, zlib_err=%d", zlib_err);
		return 1;
	}
	return 0;
}
