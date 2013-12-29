#include <stddef.h>

// define the light versions of the functions if TW_ERROR isn't set
static inline void tw_error_set_impl(const char *file, int line, const char *function, int errno, const char *fmt, ...)
{
	va_list va_args;
	va_start(va_args, fmt);
	tw_error_set_impl_v(file, line, function, errno, fmt, va_args);
	va_end(va_args);
}

static inline void tw_error_set_impl_v(const char *file, int line, const char *function, int errno, const char *fmt, va_list va_args)
{
	tw_error_errno_set(errno);
#ifdef TW_ERROR
	tw_error_msg_set_v(file, line, function, errno, fmt, va_args);
#else
	(void)file; (void)line; (void)function; (void)fmt; (void)va_args;
#endif
}

static inline void tw_error_msg_set(const char *file, int line, const char *function, int errno, const char *fmt, ...)
{
	va_list va_args;
	va_start(va_args, fmt);
	tw_error_msg_set_v(file, line, function, errno, fmt, va_args);
	va_end(va_args);
}

static inline const char *tw_error_string()
{
	return tw_error_string_impl();
}

static inline void tw_error_clear()
{
	tw_error_errno_clear();
#ifdef TW_ERROR
	tw_error_msg_clear();
#endif
}
