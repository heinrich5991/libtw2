extern crate datafile = "datafile_raw";

use datafile::DatafileReader;
use datafile::DatafileBuffer;

use std::io::File;

fn main() {
	let file = box File::open(&Path::new("../dm1.map")).unwrap();
	let dfr = match DatafileReader::read(file) {
		Ok(Ok(x)) => x,
		Ok(Err(x)) => fail!("datafile error {}", x),
		Err(x) => fail!("IO error {}", x),
	};
	//println!("{:?}", df);
	dfr.debug_dump();

	let _dfb = match DatafileBuffer::from_datafile(&dfr) {
		Some(x) => x,
		None => fail!("datafile error ..."),
	};
}
