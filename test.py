import datafile

from collections import defaultdict

def check_versions(df):
	result = []
	if len(df.types[0]) < 1:
		result.append('no version')
		return result
	if len(df.types[0]) > 1:
		result.append('multiple versions')
	try:
		version = df.types[0][0]
	except IndexError:
		result.append('version id not 1')
		return result
	if len(version.data) < 1:
		result.append('version too small')
		return result
	if len(version.data) > 1:
		result.append('version bigger than expected')
	if version.data[0] != 1:
		result.append('version not 1')
	result.append(version.data[0])
	return result

#struct CMapItemImage_v1
#{
#	int m_Version;
#	int m_Width;
#	int m_Height;
#	int m_External;
#	int m_ImageName;
#	int m_ImageData;
#} ;

#struct CMapItemImage : public CMapItemImage_v1
#{
#	enum { CURRENT_VERSION=2 };
#	int m_Format;
#};

def check_images(df):
	for image_item in df.types[2]:
		if 0 <= image_item.data[3] <= 1:
			if image_item.data[3]:
				continue
		else:
			print(df)
			pass#print("<what?>")
		print(image_item.data[3])
		name_index = image_item.data[4]
		try:
			name = df.data[name_index]
		except datafile.DatafileDataUncompressError:
			name = "<none>"
		#print(name)
	return []

def main():
	import argparse
	p = argparse.ArgumentParser()
	p.add_argument('filenames', metavar="DATAFILE", type=str, nargs='+', help="a datafile to be processed")
	p.add_argument('-s', '--summary', action='store_true', help="show summary")
	p.add_argument('-i', '--images', action='store_true', help="extract information about images")
	p.add_argument('-v', '--versions', action='store_true', help="extract information about versions")
	args = p.parse_args()

	tasks = []
	if args.images:
		tasks.append('images')
	if args.versions:
		tasks.append('versions')

	do_tasks = {'images': check_images, 'versions': check_versions}

	results = {}

	for task in tasks:
		results[task] = defaultdict(lambda: set())

	errors = defaultdict(lambda: set())
	versions = defaultdict(lambda: set())
	images = defaultdict(lambda: set())

	for filename in args.filenames:
		true_filename = filename
		filename = filename.encode('utf-8', errors='ignore').decode('utf-8')

		try:
			df = datafile.Datafile(true_filename)
		except datafile.DatafileError as e:
			errors[e.__class__].add(filename)
			print("{}: {}".format(filename, repr(e)))
		else:
			try:
				for task in tasks:
					for result in do_tasks[task](df):
						results[task][result].add(filename)
			finally:
				df.close()

	if args.summary:
		print()
		print("Error statistics:")

		for err, filenames in errors.items():
			print("### {}: {}, {}".format(err.__name__, len(filenames), " ".join(sorted(filenames))))

		print()
		print("Results:")
		for task, result in results.items():
			print("# {}:".format(task))
			for desc, filenames in sorted(result.items(), key=lambda x: len(x[1]), reverse=True):
				print("### {}: {}, {}".format(desc, len(filenames), " ".join(sorted(filenames)[:20])))

if __name__ == '__main__':
	import sys
	sys.exit(main())
