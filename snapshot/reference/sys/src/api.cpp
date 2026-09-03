#include "api.h"

#include "ddnet/snapshot.h"

#include <new>

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

struct snapshotdelta
{
	CSnapshotDelta Delta;
};

extern "C" size_t snapshotdelta_size(void)
{
	return sizeof(struct snapshotdelta);
}

extern "C" void snapshotdelta_init(struct snapshotdelta *snapshotdelta)
{
	new(snapshotdelta) struct snapshotdelta;
}

extern "C" void snapshotdelta_set_static_size(struct snapshotdelta *snapshotdelta,
		uint16_t type, size_t len)
{
	snapshotdelta->Delta.SetStaticsize(type, len * sizeof(int32_t));
}

extern "C" size_t snapshotdelta_create(struct snapshotdelta *snapshotdelta,
		const int32_t *from, const int32_t *to, int32_t (*delta)[16384])
{
	return snapshotdelta->Delta.CreateDelta((const CSnapshot *)from, (const CSnapshot *)to, delta) / sizeof(int32_t);
}

extern "C" size_t snapshotdelta_unpack(struct snapshotdelta *snapshotdelta,
		const int32_t *from, int32_t (*to)[16384], int32_t *delta, size_t delta_len)
{
	return snapshotdelta->Delta.UnpackDelta((const CSnapshot *)from, (CSnapshotBuffer *)to, delta, delta_len * sizeof(int32_t)) / sizeof(int32_t);
}
