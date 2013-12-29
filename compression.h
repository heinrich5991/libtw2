#include <stddef.h>
#include <stdint.h>

uint32_t tw_comp_crc(uint32_t crc, void *buf, size_t length);

int tw_comp_uncompress(void *dst, size_t *dst_size, const void *src, size_t src_size);

enum
{
	TW_ERRNO_COMP=400,
	TW_ERRNO_COMP_ERROR,
};
