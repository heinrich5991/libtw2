#!/usr/bin/env python
from datafile import Datafile, DatafileError

import argparse

def do(filename):
	try:
		with Datafile(filename) as df:
			pass
	except DatafileError as dfe:
		print("{}: {}".format(filename, dfe))


if __name__ == '__main__':
	p = argparse.ArgumentParser()
	p.add_argument('filenames', metavar='datafile', type=str, nargs='+', help='the datafiles to be processed')
	args = p.parse_args()
	for filename in args.filenames:
		do(filename)
