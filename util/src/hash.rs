use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::num::Wrapping;

pub struct FnvHasher {
    hash: Wrapping<u64>,
}

impl Default for FnvHasher {
    fn default() -> FnvHasher {
        FnvHasher {
            hash: Wrapping(14695981039346656037),
        }
    }
}

impl Hasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash.0 = self.hash.0 ^ (*byte as u64);
            self.hash = self.hash * Wrapping(1099511628211);
        }
    }

    fn finish(&self) -> u64 {
        self.hash.0
    }
}

pub type FnvBuildHasher = BuildHasherDefault<FnvHasher>;
pub type FnvHashMap<T, U> = HashMap<T, U, FnvBuildHasher>;
pub type FnvHashSet<T> = HashSet<T, FnvBuildHasher>;
