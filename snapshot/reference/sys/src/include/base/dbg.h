#ifndef BASE_DBG_H_SHIM
#define BASE_DBG_H_SHIM
#define PRIzu "zu"
#include <stdlib.h>
static inline void dbg_assert(bool test, const char *fmt, ...)
{
	(void)fmt;
	if(!test)
	{
		abort();
	}
}
static inline void dbg_msg(const char *sys, const char *fmt, ...)
{
	(void)sys;
	(void)fmt;
}
#endif // BASE_DBG_H_SHIM
