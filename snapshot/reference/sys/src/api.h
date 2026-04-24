#ifndef SNAPSHOT_API_H
#define SNAPSHOT_API_H

#include <stddef.h>
#include <stdint.h>

struct snapshotbuilder;

extern "C" size_t snapshotbuilder_size(void);

extern "C" void snapshotbuilder_init(struct snapshotbuilder *snapshotbuilder);
extern "C" void snapshotbuilder_add_item(struct snapshotbuilder *snapshotbuilder,
		uint16_t type, uint16_t id, const int32_t *data, size_t data_len);
extern "C" size_t snapshotbuilder_finish(struct snapshotbuilder *snapshotbuilder,
		int32_t (*buffer)[16384]);

struct snapshotdelta;

extern "C" size_t snapshotdelta_size(void);
extern "C" void snapshotdelta_init(struct snapshotdelta *snapshotdelta);
extern "C" void snapshotdelta_set_static_size(struct snapshotdelta *snapshotdelta,
		uint16_t type, size_t len);
extern "C" size_t snapshotdelta_create(struct snapshotdelta *snapshotdelta,
		const int32_t *from, const int32_t *to, int32_t (*delta)[16384]);
extern "C" size_t snapshotdelta_unpack(struct snapshotdelta *snapshotdelta,
		const int32_t *from, int32_t (*to)[16384], int32_t *delta, size_t delta_len);

#endif

