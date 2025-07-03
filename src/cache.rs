use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use nu_protocol::Span;
use chrono::{DateTime, Utc};

/// Thread-safe cache for storing Nickel plugin objects
#[derive(Debug, Clone, Default)]
pub struct NickelCache {
    inner: Arc<Mutex<HashMap<Uuid, CachedNickelValue>>>,
}

/// A cached Nickel value with metadata
#[derive(Debug, Clone)]
pub struct CachedNickelValue {
    pub uuid: Uuid,
    pub value: NickelPluginObject,
    pub created: DateTime<Utc>,
    pub span: Span,
    pub reference_count: i16,
}

/// Polymorphic storage for different types of Nickel objects
/// Uses serialization to ensure thread safety
#[derive(Debug, Clone)]
pub enum NickelPluginObject {
    /// Serializable JSON value
    JsonValue(serde_json::Value),
    /// Serialized Nickel term - stored as JSON for thread safety
    SerializedNickelTerm {
        /// The term serialized as JSON (when possible)
        json_representation: Option<serde_json::Value>,
        /// Source code representation
        source_code: String,
        /// Type information as string
        type_info: String,
    },
    /// Evaluated result
    EvaluatedValue {
        json: serde_json::Value,
        source_code: Option<String>,
    },
}

impl NickelCache {
    /// Insert a JSON value into the cache and return its UUID
    pub fn insert_json(&self, value: serde_json::Value, span: Span) -> Uuid {
        let id = Uuid::new_v4();
        let cached_value = CachedNickelValue {
            uuid: id,
            value: NickelPluginObject::JsonValue(value),
            created: Utc::now(),
            span,
            reference_count: 1,
        };
        let mut cache = self.inner.lock().unwrap();
        cache.insert(id, cached_value);
        id
    }

    /// Insert a Nickel term representation into the cache and return its UUID
    pub fn insert_nickel_term(
        &self, 
        source_code: String,
        json_representation: Option<serde_json::Value>,
        type_info: String,
        span: Span
    ) -> Uuid {
        let id = Uuid::new_v4();
        let cached_value = CachedNickelValue {
            uuid: id,
            value: NickelPluginObject::SerializedNickelTerm {
                json_representation,
                source_code,
                type_info,
            },
            created: Utc::now(),
            span,
            reference_count: 1,
        };
        let mut cache = self.inner.lock().unwrap();
        cache.insert(id, cached_value);
        id
    }

    /// Insert an evaluated value into the cache
    pub fn insert_evaluated(
        &self, 
        json: serde_json::Value, 
        source_code: Option<String>,
        span: Span
    ) -> Uuid {
        let id = Uuid::new_v4();
        let cached_value = CachedNickelValue {
            uuid: id,
            value: NickelPluginObject::EvaluatedValue {
                json,
                source_code,
            },
            created: Utc::now(),
            span,
            reference_count: 1,
        };
        let mut cache = self.inner.lock().unwrap();
        cache.insert(id, cached_value);
        id
    }

    /// Get a cached value by UUID
    pub fn get(&self, id: &Uuid) -> Option<CachedNickelValue> {
        let cache = self.inner.lock().unwrap();
        cache.get(id).cloned()
    }

    /// Increment reference count for a cached value
    pub fn increment_ref(&self, id: &Uuid) {
        let mut cache = self.inner.lock().unwrap();
        if let Some(cached_value) = cache.get_mut(id) {
            cached_value.reference_count += 1;
        }
    }

    /// Decrement reference count for a cached value, removing if it reaches 0
    pub fn decrement_ref(&self, id: &Uuid) -> bool {
        let mut cache = self.inner.lock().unwrap();
        if let Some(cached_value) = cache.get_mut(id) {
            cached_value.reference_count -= 1;
            if cached_value.reference_count <= 0 {
                cache.remove(id);
                return true; // Value was removed
            }
        }
        false // Value still exists
    }

    /// Remove a cached item by UUID
    pub fn remove(&self, id: &Uuid) -> Option<CachedNickelValue> {
        let mut cache = self.inner.lock().unwrap();
        cache.remove(id)
    }

    /// Get the number of cached items
    pub fn len(&self) -> usize {
        let cache = self.inner.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.inner.lock().unwrap();
        cache.is_empty()
    }

    /// Clean up old unused cache entries
    pub fn cleanup_old_entries(&self, max_age_hours: i64) {
        let mut cache = self.inner.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        cache.retain(|_, cached_value| {
            cached_value.reference_count > 0 || cached_value.created > cutoff
        });
    }
}

impl CachedNickelValue {
    /// Get the JSON value if this is a JSON type
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match &self.value {
            NickelPluginObject::JsonValue(json) => Some(json),
            NickelPluginObject::EvaluatedValue { json, .. } => Some(json),
            NickelPluginObject::SerializedNickelTerm { json_representation: Some(json), .. } => Some(json),
            _ => None,
        }
    }

    /// Get the source code if available
    pub fn as_source_code(&self) -> Option<&String> {
        match &self.value {
            NickelPluginObject::SerializedNickelTerm { source_code, .. } => Some(source_code),
            NickelPluginObject::EvaluatedValue { source_code: Some(code), .. } => Some(code),
            _ => None,
        }
    }

    /// Get the object type as a string for display
    pub fn object_type(&self) -> &'static str {
        match &self.value {
            NickelPluginObject::JsonValue(_) => "JsonValue",
            NickelPluginObject::SerializedNickelTerm { .. } => "NickelTerm", 
            NickelPluginObject::EvaluatedValue { .. } => "EvaluatedValue",
        }
    }

    /// Check if this value can be evaluated to JSON
    pub fn has_json_representation(&self) -> bool {
        self.as_json().is_some()
    }
}