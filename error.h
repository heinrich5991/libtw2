#include <stdarg.h>

#define tw_error_set(errno, ...) tw_error_set_impl(__FILE__, __LINE__, __FUNCTION__, (errno), __VA_ARGS__)
#define tw_error_set_v(errno, va_args) tw_error_set_impl_v(__FILE__, __LINE__, __FUNCTION__, (errno), (va_args))

static const char *tw_error_string(void);
static void tw_error_clear(void);

static void tw_error_set_impl(const char *file, int line, const char *function, int errno, const char *fmt, ...);
static void tw_error_set_impl_v(const char *file, int line, const char *function, int errno, const char *fmt, va_list va_args);

int tw_error_errno(void);
void tw_error_errno_set(int errno);
void tw_error_errno_clear(void);

static void tw_error_msg_set(const char *file, int line, const char *function, int errno, const char *fmt, ...);
void tw_error_msg_set_v(const char *file, int line, const char *function, int errno, const char *fmt, va_list va_args);
void tw_error_msg_clear(void);
const char *tw_error_string_impl(void);

enum
{
	TW_ERRNO_NONE=0,

	TW_ERRNO_GENERAL=100,
	TW_ERRNO_TYPEERROR,
	TW_ERRNO_NOTIMPLEMENTED,
	TW_ERRNO_OUTOFRANGE,
};

#include "error_inlines.h"
