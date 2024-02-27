use std::collections::hash_map;

/// Convenience trait for getting the `occupied` or `vacant` variant out of a
/// hashmap's entry.
pub trait HashMapEntryIntoInner<'a> {
    type Key;
    type Value;
    fn into_occupied(
        self,
    ) -> Option<
        hash_map::OccupiedEntry<
            'a,
            <Self as HashMapEntryIntoInner<'a>>::Key,
            <Self as HashMapEntryIntoInner<'a>>::Value,
        >,
    >;
    fn into_vacant(
        self,
    ) -> Option<
        hash_map::VacantEntry<
            'a,
            <Self as HashMapEntryIntoInner<'a>>::Key,
            <Self as HashMapEntryIntoInner<'a>>::Value,
        >,
    >;
}

impl<'a, K, V> HashMapEntryIntoInner<'a> for hash_map::Entry<'a, K, V> {
    type Key = K;
    type Value = V;
    fn into_occupied(self) -> Option<hash_map::OccupiedEntry<'a, K, V>> {
        match self {
            hash_map::Entry::Occupied(o) => Some(o),
            hash_map::Entry::Vacant(_) => None,
        }
    }
    fn into_vacant(self) -> Option<hash_map::VacantEntry<'a, K, V>> {
        match self {
            hash_map::Entry::Occupied(_) => None,
            hash_map::Entry::Vacant(v) => Some(v),
        }
    }
}
