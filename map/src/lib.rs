
#![feature(phase)]

extern crate datafile;

#[phase(plugin, link)]
extern crate map_macros;

use datafile::DatafileBuffer;

pub struct TeeworldsMap {
	df: DatafileBuffer,
}

impl TeeworldsMap {
	pub fn from_datafile(df: DatafileBuffer) -> TeeworldsMap {
		TeeworldsMap { df: df }
	}
}


