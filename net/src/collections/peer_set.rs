use super::peer_map;
use super::PeerMap;
use crate::net::PeerId;
use std::fmt;
use std::iter::FromIterator;

#[derive(Clone, Default)]
pub struct PeerSet {
    set: PeerMap<()>,
}

impl fmt::Debug for PeerSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl PeerSet {
    pub fn new() -> PeerSet {
        PeerSet {
            set: PeerMap::new(),
        }
    }
    pub fn with_capacity(cap: usize) -> PeerSet {
        PeerSet {
            set: PeerMap::with_capacity(cap),
        }
    }
    pub fn clear(&mut self) {
        self.set.clear()
    }
    pub fn len(&self) -> usize {
        self.set.len()
    }
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
    pub fn iter(&self) -> Iter {
        Iter(self.set.iter())
    }
    pub fn drain(&mut self) -> Drain {
        Drain(self.set.drain())
    }
    pub fn insert(&mut self, pid: PeerId) -> bool {
        self.set.insert(pid, ()).is_none()
    }
    pub fn remove(&mut self, pid: PeerId) {
        self.set.remove(pid)
    }
    pub fn contains(&mut self, pid: PeerId) -> bool {
        self.set.contains_key(pid)
    }
}

impl FromIterator<PeerId> for PeerSet {
    fn from_iter<I>(iter: I) -> PeerSet
    where
        I: IntoIterator<Item = PeerId>,
    {
        PeerSet {
            set: FromIterator::from_iter(iter.into_iter().map(|pid| (pid, ()))),
        }
    }
}

impl Extend<PeerId> for PeerSet {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PeerId>,
    {
        self.set.extend(iter.into_iter().map(|pid| (pid, ())))
    }
}

impl<'a> IntoIterator for &'a PeerSet {
    type Item = PeerId;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

pub struct Iter<'a>(peer_map::Iter<'a, ()>);
pub struct Drain<'a>(peer_map::Drain<'a, ()>);

impl<'a> Iterator for Iter<'a> {
    type Item = PeerId;
    fn next(&mut self) -> Option<PeerId> {
        self.0.next().map(|(pid, &())| pid)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> Iterator for Drain<'a> {
    type Item = PeerId;
    fn next(&mut self) -> Option<PeerId> {
        self.0.next().map(|(pid, ())| pid)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
