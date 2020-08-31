/*
pub fn write_datafile<T:Datafile,W:Writer>(df: &T, writer: &mut W) -> Result<IoResult<()>,()> {
    let compressed_data: Vec<Vec<u8>> = try!(result::collect(df.data_iter().map(|maybe_x| maybe_x.map(|x| {
        zlib::compress_vec(x).unwrap()
        (item_type.start, item_type.num)
    }
}

/*
pub fn write_datafile<T:Datafile,W:Writer>(df: &T, writer: &mut W) -> Result<IoResult<()>,()> {
    let compressed_data: Vec<Vec<u8>> = result::collect(df.data_iter().map(|maybe_x| maybe_x.map(|x| {
        zlib::compress_vec(x).unwrap()
    })))?;

    let size_items = df.items().fold(0, |s, i| {
        s + i.data.len() * mem::size_of::<i32>() + mem::size_of::<DatafileItemHeader>()
    });

    let size_data = compressed_data.iter().fold(0, |s, d| s + d.len());

    DatafileHeaderVersion {
        magic: DATAFILE_MAGIC,
        version: 4,
    }.write(writer)?;

    DatafileHeader {
        _size: unimplemented!(),
        _swaplen: unimplemented!(),
        num_item_types: df.item_types().len(),
        num_items: df.items().len(),
        num_data: df.data_iter().len(),
        size_items: size_items,
        size_data: size_data,
    }.write(writer)?;

    for &type_id in df.item_types() {
        let (start, num) = df.item_type_indexes_start_num(type_id);
        DatafileItemType {
            type_id: type_id.as_i32().unwrap(),
            start: start.as_i32().unwrap(),
            num: num.as_i32().unwrap(),
        }.write(writer)?;
    }

    for DatafileItem { type_id, id, data } in df.items() {
        DatafileItemHeader::new(type_id, id, data.len()).write(writer)?;
    }
    unimplemented!();
    Ok(Ok(()))
}
*/
*/
