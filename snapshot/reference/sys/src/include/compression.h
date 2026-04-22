#ifndef COMPRESSION_H_SHIM
#define COMPRESSION_H_SHIM
#include <stdlib.h>
class CVariableInt
{
public:
	enum
	{
		MAX_BYTES_PACKED = 5,
	};
	static unsigned char *Pack(unsigned char *pDst, int i, int DstSize)
	{
		(void)pDst;
		(void)i;
		(void)DstSize;
		abort();
	}
};
#endif // COMPRESSION_H_SHIM
