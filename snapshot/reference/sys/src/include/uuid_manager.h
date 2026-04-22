#ifndef UUID_MANAGER_H_SHIM
#define UUID_MANAGER_H_SHIM
#include <stdlib.h>
constexpr int OFFSET_UUID = 1 << 16;

struct CUuid
{
	unsigned char m_aData[16];
};

class CUuidManager
{
public:
	CUuid GetUuid(int Id) { (void)Id; abort(); }
	int LookupUuid(CUuid Uuid) { (void)Uuid; abort(); }
};

static CUuidManager g_UuidManager;
#endif // UUID_MANAGER_H_SHIM
