use libtw2_common::unwrap_or_return;
use std::default::Default;
use std::iter;
use std::marker::PhantomData;
use std::ops;
use std::slice;

pub trait Index: Copy {
    fn to_usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
}

impl Index for usize {
    fn to_usize(self) -> usize {
        self
    }
    fn from_usize(val: usize) -> usize {
        val
    }
}

#[derive(Clone)]
pub struct VecMap<I: Index, T> {
    pub vec: Vec<Option<T>>,
    pub marker: PhantomData<I>,
}

impl<I: Index, T> Default for VecMap<I, T> {
    fn default() -> VecMap<I, T> {
        VecMap {
            vec: vec![],
            marker: PhantomData,
        }
    }
}

impl<I: Index, T> ops::Index<I> for VecMap<I, T> {
    type Output = T;
    fn index(&self, index: I) -> &T {
        self.vec[index.to_usize()].as_ref().unwrap()
    }
}

impl<I: Index, T> ops::IndexMut<I> for VecMap<I, T> {
    fn index_mut(&mut self, index: I) -> &mut T {
        self.vec[index.to_usize()].as_mut().unwrap()
    }
}

pub type Iter<'a, I, T> = iter::FilterMap<
    iter::Enumerate<slice::Iter<'a, Option<T>>>,
    fn((usize, &Option<T>)) -> Option<(I, &T)>,
>;
impl<I: Index, T> VecMap<I, T> {
    pub fn push(&mut self, element: T) -> I {
        let index = self.vec.len();
        self.vec.push(Some(element));
        Index::from_usize(index)
    }
    pub fn iter(&self) -> Iter<I, T> {
        fn indexify_filter<I: Index, T>((idx, elem): (usize, &Option<T>)) -> Option<(I, &T)> {
            let elem = unwrap_or_return!(elem.as_ref(), None);
            Some((Index::from_usize(idx), elem))
        }
        self.vec
            .iter()
            .enumerate()
            .filter_map(indexify_filter::<I, T>)
    }
}
