#include "error.h"

#include "common.h"

#include <assert.h>
#include <stddef.h>

typedef struct tw_error
{
	char file[64];
	int line;
	char function[64];
	char msg[128];
	int errno;
} tw_error;

typedef struct tw_error_threadlocals
{
	tw_error errors[32];
	int num_errors;
	char msg[1024];
	int msg_constructed;
	int errno;
} tw_error_threadlocals;

static __thread tw_error_threadlocals tls;


void tw_error_msg_set_v(const char *file, int line, const char *function, int errno, const char *fmt, va_list va_args)
{
	// just copy everything into our local storage
	tw_error error;
	if(file)
		tw_str_copy(error.file, file, sizeof(error.file));
	else
		error.file[0] = 0;
	error.line = line;
	if(function)
		tw_str_copy(error.function, function, sizeof(error.function));
	else
		error.function[0] = 0;
	error.errno = errno;
	if(fmt)
		tw_str_format_v(error.msg, sizeof(error.msg), fmt, va_args);
	else
		error.msg[0] = 0;
	tls.errors[tls.num_errors] = error;
	tls.num_errors++;
}

void tw_error_errno_set(int errno)
{
	assert(errno != 0 && "can't set errno to 0");
	tls.errno = errno;
}

int tw_error_errno()
{
	return tls.errno;
}

void tw_error_errno_clear()
{
	tls.errno = 0;
}

void tw_error_msg_clear()
{
	tls.num_errors = 0;
	tls.msg_constructed = 0;
}

static void tw_error_msg_construct()
{
	// no errors with string? return the stringified errno
	if(tls.num_errors == 0)
	{
		tw_str_format(tls.msg, sizeof(tls.msg), "unknown error (%d)", tls.errno);
		tls.msg_constructed = 1;
		return;
	}

	tls.msg[0] = 0; // make msg the empty string

	// construct the message the following way
	// "outer func error: inner func error: innermost func error"
	/*int i;
	for(i = tls.num_errors - 1; i >= 0; i--)
	{
		tw_error *error = &tls.errors[i];
		if(error->msg[0] != 0)
		{
			// if we aren't at the beginning of the constructed
			// string, insert a ": "
			if(tls.msg[0] != 0)
				tw_str_append(tls.msg, ": ", sizeof(tls.msg));
			tw_str_append(tls.msg, error->msg, sizeof(tls.msg));
		}
	}*/
	tw_str_copy(tls.msg, tls.errors[tls.num_errors - 1].msg, sizeof(tls.msg));

	tls.msg_constructed = 1;
}

const char *tw_error_string_impl()
{
	if(tls.errno == 0)
		return NULL;

	if(tls.msg_constructed == 0)
		tw_error_msg_construct();
	assert(tls.msg_constructed != 0 && "msg construct didn't construct msg");

	return tls.msg;
}

