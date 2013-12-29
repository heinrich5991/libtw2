#include "io.h"

#include "error.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct tw_io
{
	FILE *file;
};

tw_io *tw_io_open(const char *filename, const char *mode)
{
	tw_io io;

	errno = 0;
	io.file = fopen(filename, mode);
	if(io.file == NULL)
	{
		tw_error_set(TW_ERRNO_IO_ERROR, "could not open file, name=\"%s\" errno=%d msg=\"%s\"", filename, errno, strerror(errno));
		return NULL;
	}

	tw_io *ret = malloc(sizeof(*ret));
	*ret = io;
	return ret;
}

int tw_io_close(tw_io *io)
{
	tw_io io_c = *io;
	free(io);
	io = NULL;

	if(fclose(io_c.file) != 0)
	{
		tw_error_set(TW_ERRNO_IO_ERROR, "could not close, errno=%d msg=\"%s\"", errno, strerror(errno));
		return 1;
	}
	return 0;
}

size_t tw_io_read(tw_io *io, void *buffer, size_t size)
{
	size_t read = fread(buffer, 1, size, io->file);
	if(read < size)
	{
		if(ferror(io->file) != 0)
			tw_error_set(TW_ERRNO_IO_ERROR, "read failed");
		else if(feof(io->file) != 0)
			tw_error_set(TW_ERRNO_IO_EOF, "EOF reached");
	}
	return read;
}

size_t tw_io_write(tw_io *io, const void *buffer, size_t size)
{
	size_t wrote = fwrite(buffer, 1, size, io->file);
	if(wrote < size && ferror(io->file) != 0)
		tw_error_set(TW_ERRNO_IO_ERROR, "write failed");
	return wrote;
}

long tw_io_tell(tw_io *io)
{
	long tell;
	errno = 0;
	tell = ftell(io->file);
	if(tell == EOF)
	{
		tw_error_set(TW_ERRNO_IO_ERROR, "tell failed: %s", strerror(errno));
		return -1;
	}
	return tell;
}

int tw_io_seek(tw_io *io, long offset)
{
	errno = 0;
	if(fseek(io->file, offset, SEEK_SET) != 0)
	{
		tw_error_set(TW_ERRNO_IO_ERROR, "seek failed: %s", strerror(errno));
		return 1;
	}
	return 0;
}

int tw_io_seek_end(tw_io *io)
{
	errno = 0;
	if(fseek(io->file, 0, SEEK_END) != 0)
	{
		tw_error_set(TW_ERRNO_IO_ERROR, "seek failed: %s", strerror(errno));
		return 1;
	}
	return 0;
}

int tw_io_flush(tw_io *io)
{
	errno = 0;
	if(fflush(io->file) != 0)
	{
		tw_error_set(TW_ERRNO_IO_ERROR, "flush failed: %s", strerror(errno));
		return 1;
	}
	return 0;
}
