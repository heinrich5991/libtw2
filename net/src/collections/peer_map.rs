use linear_map::LinearMap;
use linear_map;
use net::PeerId;
use std::fmt;
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
    pub fn iter(&self) -> Iter<T> {
        Iter(self.map.iter())
    }
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.map.iter_mut())
    }
    pub fn remove(&mut self, pid: PeerId) {
        self.map.remove(&pid).unwrap_or_else(|| panic!("invalid pid"));
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

pub struct Iter<'a, T: 'a>(linear_map::Iter<'a, PeerId, T>);
pub struct IterMut<'a, T: 'a>(linear_map::IterMut<'a, PeerId, T>);

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

pub struct OccupiedEntry<'a, T: 'a>(linear_map::OccupiedEntry<'a, PeerId, T>);
pub struct VacantEntry<'a, T: 'a>(linear_map::VacantEntry<'a, PeerId, T>);

pub enum Entry<'a, T: 'a> {
    Occupied(OccupiedEntry<'a, T>),
    Vacant(VacantEntry<'a, T>),
}

impl<'a, T: 'a> VacantEntry<'a, T> {
    pub fn insert(self, value: T) -> &'a mut T {
        self.0.insert(value)
    }
}
