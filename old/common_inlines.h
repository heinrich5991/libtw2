#include <stdio.h>
#include <string.h>

static inline void tw_str_copy(char *dst, const char *src, size_t dst_size)
{
	strncpy(dst, src, dst_size);
	dst[dst_size - 1] = 0; // null termination
}

static inline void tw_str_append(char *dst, const char *src, size_t dst_size)
{
	unsigned len = strlen(dst);
	if(len < dst_size)
		strncpy(dst + len, src, dst_size - len);
	dst[dst_size - 1] = 0;
}

static inline int tw_str_comp(const char *str1, const char *str2)
{
	return strcmp(str1, str2);
}

static inline size_t tw_str_length(const char *str)
{
	return strlen(str);
}

static inline void tw_mem_copy(void *dst, const void *src, size_t size)
{
	memcpy(dst, src, size);
}

static inline void tw_mem_move(void *dst, const void *src, size_t size)
{
	memmove(dst, src, size);
}

static inline int tw_mem_comp(const void *mem1, const void *mem2, size_t size)
{
	return memcmp(mem1, mem2, size);
}

static inline void tw_mem_zero(void *mem, size_t size)
{
	memset(mem, 0, size);
}

static inline void tw_str_format(char *dst, size_t dst_size, const char *fmt, ...)
{
	va_list va_args;
	va_start(va_args, fmt);
	tw_str_format_v(dst, dst_size, fmt, va_args);
	va_end(va_args);
}

static inline void tw_str_format_v(char *dst, size_t dst_size, const char *fmt, va_list va_args)
{
	vsnprintf(dst, dst_size, fmt, va_args);
	dst[dst_size - 1] = 0; // null termination
}

static inline void tw_endian_tolittle(void *data, size_t size, int count)
{
	(void)data; (void)size; (void)count;
}

static inline void tw_endian_tobig(void *data, size_t size, int count)
{
	tw_endian_swap(data, size, count);
}

static inline void tw_endian_fromlittle(void *data, size_t size, int count)
{
	(void)data; (void)size; (void)count;
}

static inline void tw_endian_frombig(void *data, size_t size, int count)
{
	tw_endian_swap(data, size, count);
}
