use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use nickel_lang_core::term::Term;

/// Cache for storing Nickel values and evaluations
#[derive(Debug, Clone, Default)]
pub struct NickelCache {
    inner: Arc<Mutex<HashMap<Uuid, CachedItem>>>,
}

#[derive(Debug, Clone)]
pub enum CachedItem {
    Term(Term),
    Value(nickel_lang_core::term::RichTerm),
}

impl NickelCache {
    pub fn insert_term(&self, term: Term) -> Uuid {
        let id = Uuid::new_v4();
        let mut cache = self.inner.lock().unwrap();
        cache.insert(id, CachedItem::Term(term));
        id
    }

    pub fn insert_value(&self, value: nickel_lang_core::term::RichTerm) -> Uuid {
        let id = Uuid::new_v4();
        let mut cache = self.inner.lock().unwrap();
        cache.insert(id, CachedItem::Value(value));
        id
    }

    pub fn get(&self, id: &Uuid) -> Option<CachedItem> {
        let cache = self.inner.lock().unwrap();
        cache.get(id).cloned()
    }

    pub fn remove(&self, id: &Uuid) -> Option<CachedItem> {
        let mut cache = self.inner.lock().unwrap();
        cache.remove(id)
    }

    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }
}