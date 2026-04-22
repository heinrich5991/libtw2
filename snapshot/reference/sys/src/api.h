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

#endif

