#include "api.h"

#include "ddnet/snapshot.h"

struct snapshotbuilder
{
	CSnapshotBuilder Builder;
};

extern "C" size_t snapshotbuilder_size(void)
{
	return sizeof(struct snapshotbuilder);
}

extern "C" void snapshotbuilder_init(struct snapshotbuilder *snapshotbuilder)
{
	snapshotbuilder->Builder.Init(false);
}

extern "C" void snapshotbuilder_add_item(struct snapshotbuilder *snapshotbuilder,
		uint16_t type, uint16_t id, const int32_t *data, size_t data_len)
{
	snapshotbuilder->Builder.NewItem(type, id, data, data_len * sizeof(int32_t));
}

extern "C" size_t snapshotbuilder_finish(struct snapshotbuilder *snapshotbuilder,
		int32_t (*buffer)[16384])
{
	return snapshotbuilder->Builder.Finish((CSnapshotBuffer *)buffer) / sizeof(int32_t);
}
