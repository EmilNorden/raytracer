use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Weak};

pub struct ContentCache<T, U> {
    items: HashMap<T, Weak<U>>
}

impl<T, U> ContentCache<T,U>
where T: Eq + Hash{
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn get(&self, key: T) -> Option<Arc<U>> {
        self.items.get(&key).map(|s| s.upgrade().unwrap())
    }

    pub fn insert(&mut self, key: T, item: U) -> Arc<U> {
        let rc = Arc::new(item);
        self.items.insert(key, Arc::downgrade(&rc));

        rc
    }
}