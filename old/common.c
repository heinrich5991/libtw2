
#include "common.h"

void tw_endian_swap(void *data, size_t size, int count)
{
	unsigned char *data_c = (unsigned char *)data;
	for(; count >= 0; count--)
	{
		unsigned char *src = (unsigned char *)data_c;
		unsigned char *dst = (unsigned char *)data_c + (size - 1);

		for(; src < dst; src++, dst--)
		{
			unsigned char tmp;
			tmp = *src;
			*src = *dst;
			*dst = tmp;
		}

		data_c += size;
	}
}

