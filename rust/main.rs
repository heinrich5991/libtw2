extern crate datafile = "datafile_raw";

use datafile::DatafileReader;

use std::io::File;

fn main() {
	let file = box File::open(&Path::new("../dm1.map")).unwrap();
	let df = match DatafileReader::read(file) {
		Ok(Ok(x)) => x,
		Ok(Err(x)) => fail!("datafile error {:?}", x),
		Err(x) => fail!("IO error {:?}", x),
	};
	//println!("{:?}", df);
	df.debug_dump();
}
