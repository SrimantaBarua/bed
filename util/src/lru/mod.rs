use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

mod ll;

struct KV<K, V> {
    key: K,
    value: V,
}

/// A key-value hashmap with limited capacity, which implements LRU removal of elements. Each
/// access of an element, including insertion, marks it as the most recently used. Once the
/// map reaches capacity, the next insertion will evict the least recently used item from the
/// hash map.
pub struct LruHashMap<K, V, S = RandomState>
where
    K: Clone + Eq + Hash,
    S: BuildHasher + Default,
{
    map: HashMap<K, ll::NodePtr<KV<K, V>>, S>,
    list: ll::List<KV<K, V>>,
    capacity: usize,
}

impl<K, V> LruHashMap<K, V, RandomState>
where
    K: Clone + Eq + Hash,
{
    pub fn new(capacity: usize) -> Self {
        Self::with_hasher(capacity, RandomState::default())
    }
}

impl<K, V, S> LruHashMap<K, V, S>
where
    K: Clone + Eq + Hash,
    S: BuildHasher + Default,
{
    pub fn with_hasher(capacity: usize, hash_builder: S) -> Self {
        LruHashMap {
            map: HashMap::with_hasher(hash_builder),
            list: ll::List::new(),
            capacity,
        }
    }

    /// Inserts the `key`-`value` pair into the hash map. If an LRU item was evicted, the key-value
    /// pair that were evicted are returned. If a key was replaced, then that key-value pair is
    /// returned. Otherwise this function returns `None`.
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let mut ret = self.remove(&key).map(|v| (key.clone(), v));
        if self.len() == self.capacity {
            assert!(
                ret.is_none(),
                "if we've already removed something, we should be below capacity"
            );
            ret = self.pop_lru();
        }
        let node_ptr = self.list.push_back(KV {
            key: key.clone(),
            value,
        });
        self.map.insert(key, node_ptr);
        assert!(self.len() <= self.capacity);
        ret
    }

    /// Removes the entry for the provided key in the hash map, if present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map
            .remove(key)
            .map(|node_ptr| unsafe { self.list.remove(node_ptr) }.unwrap().value)
    }

    /// Removes the least recently used entry (if one is present), and returns the key and value.
    pub fn pop_lru(&mut self) -> Option<(K, V)> {
        self.list.pop_front().map(|kv| {
            self.map.remove(&kv.key);
            (kv.key, kv.value)
        })
    }

    /// Get an immutable reference to the value mapped for a key, if it exists in the map. Updates
    /// the entry to be the most recently used.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let list = &mut self.list;
        self.map.get(key).map(|node_ptr| unsafe {
            list.move_to_end(*node_ptr);
            &node_ptr.data().value
        })
    }

    /// Get a mutable reference to the value mapped for a key, if it exists in the map. Updates the
    /// entry to be the most recently used.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let list = &mut self.list;
        self.map.get_mut(key).map(|node_ptr| unsafe {
            list.move_to_end(*node_ptr);
            &mut node_ptr.data_mut().value
        })
    }

    /// Gets number of entries in the LRU hashmap.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru() {
        let mut map = LruHashMap::new(5);
        assert!(map.insert(1, 10).is_none());
        assert_eq!(map.get(&1), Some(&10));
        assert!(map.insert(2, 20).is_none());
        assert!(map.insert(3, 30).is_none());
        assert!(map.insert(4, 40).is_none());
        assert!(map.insert(5, 50).is_none());
        assert_eq!(map.len(), 5);
        assert_eq!(map.insert(6, 60), Some((1, 10)));
        assert_eq!(map.len(), 5);
        assert_eq!(map.get(&4), Some(&40));
        assert_eq!(map.get(&2), Some(&20));
        assert_eq!(map.insert(7, 70), Some((3, 30)));
        assert_eq!(map.pop_lru(), Some((5, 50)));
        assert_eq!(map.remove(&6), Some(60));
        assert_eq!(map.pop_lru(), Some((4, 40)));
        assert_eq!(map.get(&2), Some(&20));
        assert_eq!(map.pop_lru(), Some((7, 70)));
        assert_eq!(map.pop_lru(), Some((2, 20)));
        assert!(map.pop_lru().is_none());
        assert_eq!(map.len(), 0);
    }
}
