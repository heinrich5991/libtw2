#include <stdlib.h>

typedef struct tw_io tw_io;

tw_io *tw_io_open(const char *filename, const char *mode);
int tw_io_close(tw_io *io);
size_t tw_io_read(tw_io *io, void *buffer, size_t size);
size_t tw_io_write(tw_io *io, const void *buffer, size_t size);
long tw_io_tell(tw_io *io);
int tw_io_seek(tw_io *io, long offset);
int tw_io_seek_end(tw_io *io);
size_t io_size(tw_io *io);
int tw_io_flush(tw_io *io);

enum
{
	TW_ERRNO_IO=200,
	TW_ERRNO_IO_EOF,
	TW_ERRNO_IO_ERROR,
};
