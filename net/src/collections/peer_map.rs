use linear_map;
use linear_map::LinearMap;
use net::PeerId;
use std::fmt;
use std::iter::FromIterator;
use std::ops;

#[derive(Clone)]
pub struct PeerMap<T> {
    // TODO: Different data structure, HashMap?
    map: LinearMap<PeerId, T>,
}

impl<T> Default for PeerMap<T> {
    fn default() -> PeerMap<T> {
        PeerMap::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for PeerMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> PeerMap<T> {
    pub fn new() -> PeerMap<T> {
        PeerMap {
            map: LinearMap::new(),
        }
    }
    pub fn with_capacity(cap: usize) -> PeerMap<T> {
        PeerMap {
            map: LinearMap::with_capacity(cap),
        }
    }
    pub fn clear(&mut self) {
        self.map.clear()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn iter(&self) -> Iter<T> {
        Iter(self.map.iter())
    }
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.map.iter_mut())
    }
    pub fn keys(&self) -> Keys<T> {
        Keys(self.map.keys())
    }
    pub fn values(&self) -> Values<T> {
        Values(self.map.values())
    }
    pub fn drain(&mut self) -> Drain<T> {
        Drain(self.map.drain())
    }
    pub fn insert(&mut self, pid: PeerId, value: T) -> Option<T> {
        self.map.insert(pid, value)
    }
    pub fn remove(&mut self, pid: PeerId) {
        self.map
            .remove(&pid)
            .unwrap_or_else(|| panic!("invalid pid"));
    }
    pub fn get(&self, pid: PeerId) -> Option<&T> {
        self.map.get(&pid)
    }
    pub fn get_mut(&mut self, pid: PeerId) -> Option<&mut T> {
        self.map.get_mut(&pid)
    }
    pub fn entry(&mut self, pid: PeerId) -> Entry<T> {
        match self.map.entry(pid) {
            linear_map::Entry::Occupied(o) => Entry::Occupied(OccupiedEntry(o)),
            linear_map::Entry::Vacant(v) => Entry::Vacant(VacantEntry(v)),
        }
    }
    pub fn contains_key(&mut self, pid: PeerId) -> bool {
        self.map.contains_key(&pid)
    }
}

impl<T> ops::Index<PeerId> for PeerMap<T> {
    type Output = T;
    fn index(&self, pid: PeerId) -> &T {
        self.get(pid).unwrap_or_else(|| panic!("invalid pid"))
    }
}

impl<T> ops::IndexMut<PeerId> for PeerMap<T> {
    fn index_mut(&mut self, pid: PeerId) -> &mut T {
        self.get_mut(pid).unwrap_or_else(|| panic!("invalid pid"))
    }
}

impl<T> FromIterator<(PeerId, T)> for PeerMap<T> {
    fn from_iter<I>(iter: I) -> PeerMap<T>
    where
        I: IntoIterator<Item = (PeerId, T)>,
    {
        PeerMap {
            map: FromIterator::from_iter(iter),
        }
    }
}

impl<T> Extend<(PeerId, T)> for PeerMap<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (PeerId, T)>,
    {
        self.map.extend(iter)
    }
}

impl<'a, T: 'a> IntoIterator for &'a PeerMap<T> {
    type Item = (PeerId, &'a T);
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut PeerMap<T> {
    type Item = (PeerId, &'a mut T);
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

pub struct Iter<'a, T: 'a>(linear_map::Iter<'a, PeerId, T>);
pub struct IterMut<'a, T: 'a>(linear_map::IterMut<'a, PeerId, T>);
pub struct Drain<'a, T: 'a>(linear_map::Drain<'a, PeerId, T>);
pub struct Keys<'a, T: 'a>(linear_map::Keys<'a, PeerId, T>);
pub struct Values<'a, T: 'a>(linear_map::Values<'a, PeerId, T>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (PeerId, &'a T);
    fn next(&mut self) -> Option<(PeerId, &'a T)> {
        self.0.next().map(|(&pid, e)| (pid, e))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (PeerId, &'a mut T);
    fn next(&mut self) -> Option<(PeerId, &'a mut T)> {
        self.0.next().map(|(&pid, e)| (pid, e))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = (PeerId, T);
    fn next(&mut self) -> Option<(PeerId, T)> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: 'a> Iterator for Keys<'a, T> {
    type Item = PeerId;
    fn next(&mut self) -> Option<PeerId> {
        self.0.next().cloned()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, T: 'a> Iterator for Values<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct OccupiedEntry<'a, T: 'a>(linear_map::OccupiedEntry<'a, PeerId, T>);
pub struct VacantEntry<'a, T: 'a>(linear_map::VacantEntry<'a, PeerId, T>);

pub enum Entry<'a, T: 'a> {
    Occupied(OccupiedEntry<'a, T>),
    Vacant(VacantEntry<'a, T>),
}

impl<'a, T: 'a> Entry<'a, T> {
    pub fn assert_occupied(self) -> OccupiedEntry<'a, T> {
        match self {
            Entry::Occupied(o) => o,
            Entry::Vacant(_) => panic!("invalid pid"),
        }
    }
}

impl<'a, T: 'a> OccupiedEntry<'a, T> {
    pub fn get(&self) -> &T {
        self.0.get()
    }
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
    pub fn remove(self) -> T {
        self.0.remove()
    }
}

impl<'a, T: 'a> VacantEntry<'a, T> {
    pub fn insert(self, value: T) -> &'a mut T {
        self.0.insert(value)
    }
}
