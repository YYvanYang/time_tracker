use crate::error::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

pub struct DataCache {
    cache: Mutex<LruCache<String, Vec<u8>>>,
}

impl DataCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        self.cache.lock().unwrap().put(key, value);
    }
} 