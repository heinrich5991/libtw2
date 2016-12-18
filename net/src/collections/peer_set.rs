use net::PeerId;
use std::fmt;
use super::PeerMap;
use super::peer_map;

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
        Default::default()
    }
    pub fn with_capacity(cap: usize) -> PeerSet {
        PeerSet {
            set: PeerMap::with_capacity(cap),
        }
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
