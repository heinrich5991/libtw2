#ifndef BASE_SYSTEM_H_SHIM
#define BASE_SYSTEM_H_SHIM
#include <string.h>
static inline void mem_zero(void *memory, unsigned size)
{
	memset(memory, 0, size);
}
#endif // BASE_SYSTEM_H_SHIM
