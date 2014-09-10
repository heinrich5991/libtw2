#include <stdarg.h>
#include <stddef.h>

typedef unsigned char tw_byte;

static void tw_str_copy(char *dst, const char *src, size_t dst_size);
static void tw_str_append(char *dst, const char *src, size_t dst_size);
static int tw_str_comp(const char *str1, const char *str2);
static size_t tw_str_length(const char *str);
static void tw_str_format(char *dst, size_t dst_size, const char *fmt, ...);
static void tw_str_format_v(char *dst, size_t dst_size, const char *fmt, va_list va_args);

static void tw_mem_copy(void *dst, const void *src, size_t size);
static void tw_mem_move(void *dst, const void *src, size_t size);
static int tw_mem_comp(const void *mem1, const void *mem2, size_t size);
static void tw_mem_zero(void *mem, size_t size);

void tw_endian_swap(void *data, size_t size, int count);

static void tw_endian_tolittle(void *data, size_t size, int count);
static void tw_endian_tobig(void *data, size_t size, int count);
static void tw_endian_fromlittle(void *data, size_t size, int count);
static void tw_endian_frombig(void *data, size_t size, int count);

enum
{
	TW_BUFSIZE=4096,
};

#include "common_inlines.h"
