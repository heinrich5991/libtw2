#include "datafile.h"
#include "error.h"
#include "gamemap_constants.h"

#include <stdio.h>

enum
{
	COUNT_NOVERSION=0,
	COUNT_MULTIPLEVERSIONS,
	COUNT_SMALLVERSION,
	COUNT_BIGVERSION,
	COUNT_VERSIONNOT1,
	COUNT_VERSIONIDNOT0,
	NUM_COUNTS,

	MAX_VERSIONS=1024,
};

static const char *const COUNT_NAMES[NUM_COUNTS] = {
	"no version",
	"multiple versions",
	"version too small",
	"version bigger than expected",
	"version not 1",
	"version ID not 0",
};

int main(int argc, char **argv)
{
	if(argc < 1 + 1)
	{
		fprintf(stderr, "USAGE: %s <datafile>...\n", argv[0]);
		return 1;
	}

	int counts[NUM_COUNTS] = { 0 };

	int versions[MAX_VERSIONS][2];
	int num_versions = 0;

	int i;
	for(i = 1; i < argc; i++)
	{
		const char *filename = argv[i];

		#define COUNT(count) do { counts[count]++; printf("%s: %s\n", filename, COUNT_NAMES[count]); } while(0)

		tw_error_clear();
		tw_datafile *df = tw_df_open(filename);
		if(!df)
		{
			fprintf(stderr, "%s: %d: %s\n", filename, tw_error_errno(), tw_error_string());
			continue;
		}

		int start;
		int num;
		tw_df_type_indexes(df, TW_MAP_ITEMTYPE_VERSION, &start, &num);

		if(num == 0)
			COUNT(COUNT_NOVERSION);
		else if(num > 1)
			COUNT(COUNT_MULTIPLEVERSIONS);

		int k;
		for(k = start; k < start + num; k++)
		{
			size_t size;
			int id;
			void *item = tw_df_item_read(df, k, &size, NULL, &id);

			if(size < sizeof(tw_map_item_version) % sizeof(int32_t))
			{
				COUNT(COUNT_SMALLVERSION);
				continue;
			}

			if(size > sizeof(tw_map_item_version) / sizeof(int32_t))
				COUNT(COUNT_BIGVERSION);

			if(id != 0)
				COUNT(COUNT_VERSIONIDNOT0);

			tw_map_item_version *version = item;
			if(version->version != 1)
				COUNT(COUNT_VERSIONNOT1);

			int l;
			for(l = 0; l < num_versions; l++)
			{
				if(versions[l][0] == version->version)
				{
					versions[l][1]++;
					break;
				}
			}

			// nothing found
			if(l == num_versions)
			{
				if(l < MAX_VERSIONS)
				{
					versions[l][0] = version->version;
					versions[l][1] = 1;
					num_versions++;
				}
				else
					printf("too much versions\n");
			}
		}

		//tw_df_dump(df);*/

		tw_error_clear();
		if(tw_df_close(df) != 0)
		{
			fprintf(stderr, "%s: %d: %s", filename, tw_error_errno(), tw_error_string());
			continue;
		}
	}

	for(i = 0; i < NUM_COUNTS; i++)
		printf("%s: %d\n", COUNT_NAMES[i], counts[i]);

	for(i = 0; i < num_versions; i++)
		printf("version %d: %d\n", versions[i][0], versions[i][1]);

	return 0;
}
