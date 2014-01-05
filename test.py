import datafile

from collections import defaultdict

def main():
	import argparse
	p = argparse.ArgumentParser()
	p.add_argument('filenames', metavar="DATAFILE", type=str, nargs='+', help="a datafile to be processed")
	args = p.parse_args()

	errors = defaultdict(lambda: set())
	results = defaultdict(lambda: set())
	versions = defaultdict(lambda: set())

	for filename in args.filenames:
		true_filename = filename
		try:
			filename = filename.encode('utf-8', errors='ignore').decode('utf-8')
		except UnicodeError as err:
			print(repr(filename))
			raise
		try:
			with datafile.Datafile(true_filename) as df:
				if len(df.types[0]) < 1:
					results['no version'].add(filename)
					continue
				if len(df.types[0]) > 1:
					results['multiple versions'].add(filename)

				try:
					version = df.types[0][0]
				except IndexError:
					results['version id not 0'].add(filename)
					continue

				if len(version.data) < 1:
					results['version too small'].add(filename)
					continue

				if len(version.data) > 1:
					results['version bigger than expected'].add(filename)

				if version.data[0] != 1:
					results['version not 1'].add(filename)

				versions[version.data[0]].add(filename)

		except datafile.DatafileError as e:
			errors[e.__class__].add(filename)
			print("{}: {}".format(filename, repr(e)))

	print()
	print("Error statistics:")

	for err, filenames in errors.items():
		print("### {}: {}, {}".format(err.__name__, len(filenames), " ".join(sorted(filenames))))

	print()
	print("Results:")
	for x, filenames in results.items():
		print("### {}: {}, {}".format(x, len(filenames), " ".join(sorted(filenames))))

	print()
	print("Versions:")
	for x, filenames in sorted(versions.items(), key=lambda x: len(x[1]), reverse=True):
		if x != 1:
			print("### {}: {}, {}".format(x, len(filenames), " ".join(sorted(filenames))))
		else:
			print("### {}: {}".format(x, len(filenames)))


if __name__ == '__main__':
	import sys
	sys.exit(main())
