#ifndef BASE_MEM_H_SHIM
#define BASE_MEM_H_SHIM
#include <string.h>
static inline int mem_comp(const void *a, const void *b, size_t size)
{
	return memcmp(a, b, size);
}
static inline void mem_copy(void *dst, const void *src, size_t size)
{
	memcpy(dst, src, size);
}
static inline void mem_zero(void *block, size_t size)
{
	memset(block, 0, size);
}
#endif // BASE_MEM_H_SHIM
