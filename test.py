import datafile

def main():
	import argparse
	p = argparse.ArgumentParser()
	p.add_argument('filenames', metavar="DATAFILE", type=str, nargs='+', help="a datafile to be processed")
	args = p.parse_args()

	for filename in args.filenames:
		try:
			with datafile.Datafile(filename) as df:
				pass
		except datafile.DatafileError as e:
			print("{}: {}".format(df, repr(e)))

if __name__ == '__main__':
	import sys
	sys.exit(main())
