#ifndef BASE_BYTES_H_SHIM
#define BASE_BYTES_H_SHIM
#include <stdlib.h>
static inline void uint_to_bytes_be(unsigned char *bytes, unsigned value)
{
	(void)bytes;
	(void)value;
	abort();
}
static inline unsigned bytes_be_to_uint(const unsigned char *bytes)
{
	(void)bytes;
	abort();
}
#endif // BASE_BYTES_H_SHIM
