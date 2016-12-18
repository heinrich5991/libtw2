use super::PeerMap;

#[derive(Clone, Debug, Default)]
pub struct PeerSet {
    set: PeerMap<()>,
}
