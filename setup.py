#!/usr/bin/env python
from distutils.core import setup
from distutils.extension import Extension
from Cython.Distutils import build_ext

setup(
	name="Teeworlds datafile library",
	cmdclass={'build_ext': build_ext},
	ext_modules=[Extension(
		"datafile_py",
		sources=[
			"datafile_py.pyx",
			"datafile_raw.c",
			"common.c",
		],
		libraries=["z"],
	)],
)
