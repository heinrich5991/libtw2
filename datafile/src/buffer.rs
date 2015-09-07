use common::MapIterator;
use format::ItemView;
use std::ops;

#[derive(Clone, Copy, Debug)]
struct ItemType {
    type_id: u16,
    start: usize,
    num: usize,
}

#[derive(Clone, Debug)]
struct Item {
    type_id: u16,
    id: u16,
    data: Vec<i32>,
}

pub struct Buffer {
    item_types: Vec<ItemType>,
    items: Vec<Item>,
    data: Vec<Vec<u8>>,
}

pub type DataIter<'a> = MapIterator<&'a [u8],&'a Buffer,ops::Range<usize>>;
pub type Items<'a> = MapIterator<ItemView<'a>,&'a Buffer,ops::Range<usize>>;
pub type ItemTypes<'a> = MapIterator<u16,&'a Buffer,ops::Range<usize>>;
pub type ItemTypeItems<'a> = MapIterator<ItemView<'a>,&'a Buffer,ops::Range<usize>>;

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            item_types: Vec::new(),
            items: Vec::new(),
            data: Vec::new(),
        }
    }

    fn get_item_type_index(&self, type_id: u16) -> (usize, bool) {
        for (i, &ItemType { type_id: other_type_id, .. }) in self.item_types.iter().enumerate() {
            if type_id <= other_type_id {
                return (i, type_id == other_type_id);
            }
        }
        (self.item_types.len(), false)
    }

    fn get_item_index(&self, item_type_index: usize, item_type_found: bool, id: u16) -> (usize, bool) {
        if !item_type_found {
            if item_type_index != self.item_types.len() {
                (self.item_types[item_type_index].start, false)
            } else {
                (self.items.len(), false)
            }
        } else {
            let ItemType { start, num, .. } = self.item_types[item_type_index];

            for (i, &Item { id: other_id, .. })
                in self.items[start..][..num].iter().enumerate().map(|(i, x)| (start+i, x)) {

                if id <= other_id {
                    return (i, id == other_id)
                }
            }

            (start + num, false)
        }
    }

    pub fn item_type(&self, index: usize) -> u16 {
        self.item_types.iter().nth(index).expect("Invalid type index").type_id
    }
    pub fn num_item_types(&self) -> usize {
        self.item_types.len()
    }

    pub fn item<'a>(&'a self, index: usize) -> ItemView<'a> {
        let Item { type_id, id, ref data } = self.items[index];
        ItemView {
            type_id: type_id,
            id: id,
            data: &data,
        }
    }
    pub fn num_items(&self) -> usize {
        self.items.len()
    }

    pub fn data<'a>(&'a self, index: usize) -> &'a [u8] {
        &self.data[index]
    }
    pub fn num_data(&self) -> usize {
        self.data.len()
    }

    pub fn item_type_indices(&self, type_id: u16) -> ops::Range<usize> {
        let (type_index, type_found) = self.get_item_type_index(type_id);
        if !type_found {
            return 0..0
        }
        let item_type = self.item_types[type_index];
        item_type.start..item_type.start+item_type.num
    }

    pub fn items(&self) -> Items {
        fn map_fn<'a>(i: usize, &mut self_: &mut &'a Buffer) -> ItemView<'a> {
            self_.item(i)
        }
        MapIterator::new(self, 0..self.num_items(), map_fn)
    }

    pub fn item_types(&self) -> ItemTypes {
        fn map_fn<'a>(i: usize, &mut self_: &mut &'a Buffer) -> u16 {
            self_.item_type(i)
        }
        MapIterator::new(self, 0..self.num_item_types(), map_fn)
    }

    pub fn item_type_items(&self, item_type: u16) -> ItemTypeItems {
        fn map_fn<'a>(i: usize, &mut self_: &mut &'a Buffer) -> ItemView<'a> {
            self_.item(i)
        }
        MapIterator::new(self, self.item_type_indices(item_type), map_fn)
    }

    pub fn data_iter(&self) -> DataIter {
        fn map_fn<'a>(i: usize, &mut self_: &mut &'a Buffer) -> &'a [u8] {
            self_.data(i)
        }
        MapIterator::new(self, 0..self.num_data(), map_fn)
    }

    pub fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(),()> {
        let (type_index, type_found) = self.get_item_type_index(type_id);
        let (item_index, item_found) = self.get_item_index(type_index, type_found, id);

        // if we already have an item of the given type and id,
        // return an error
        if item_found {
            return Err(());
        }

        // if there isn't a type with such an id yet, insert it
        if !type_found {
            self.item_types.insert(type_index, ItemType {
                type_id: type_id,
                start: item_index,
                num: 0,
            });
        }

        // we're going to insert an item, increase the count by one
        self.item_types[type_index].num += 1;

        // increase the starts of the following item types by one
        for t in self.item_types.iter_mut().skip(type_index + 1) {
            t.start += 1;
        }

        // actually insert the item
        self.items.insert(item_index, Item {
            type_id: type_id,
            id: id,
            data: data.to_vec(),
        });

        Ok(())
    }

    pub fn add_data(&mut self, data: Vec<u8>) -> usize {
        // add the data
        self.data.push(data);
        // return the index
        self.data.len() - 1
    }
}
