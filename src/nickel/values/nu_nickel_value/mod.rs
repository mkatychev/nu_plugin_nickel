pub mod custom_value;

use crate::{NickelPlugin, cache::CachedNickelValue};
use nu_protocol::{LabeledError, Span, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use custom_value::NuNickelValueCustomValue;

/// A wrapper for Nickel values in Nushell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuNickelValue {
    pub id: Uuid,
    pub type_name: String,
}

impl NuNickelValue {
    pub fn new(id: Uuid, type_name: String) -> Self {
        Self { id, type_name }
    }

    pub fn into_value(self, span: Span) -> Value {
        Value::custom(Box::new(NuNickelValueCustomValue::new(self)), span)
    }

    /// Cache a JSON value and create a NuNickelValue
    pub fn cache_json_value(
        plugin: &NickelPlugin,
        json_value: serde_json::Value,
        span: Span,
    ) -> Result<Value, LabeledError> {
        let id = plugin.cache.insert_json(json_value, span);
        let nu_value = NuNickelValue::new(id, "JsonValue".to_string());
        Ok(nu_value.into_value(span))
    }

    /// Cache a Nickel term representation and create a NuNickelValue
    pub fn cache_nickel_term(
        plugin: &NickelPlugin,
        source_code: String,
        json_representation: Option<serde_json::Value>,
        type_info: String,
        span: Span,
    ) -> Result<Value, LabeledError> {
        let id = plugin.cache.insert_nickel_term(source_code, json_representation, type_info, span);
        let nu_value = NuNickelValue::new(id, "NickelTerm".to_string());
        Ok(nu_value.into_value(span))
    }

    /// Cache an evaluated value and create a NuNickelValue
    pub fn cache_evaluated_value(
        plugin: &NickelPlugin,
        json: serde_json::Value,
        source_code: Option<String>,
        span: Span,
    ) -> Result<Value, LabeledError> {
        let id = plugin.cache.insert_evaluated(json, source_code, span);
        let nu_value = NuNickelValue::new(id, "EvaluatedValue".to_string());
        Ok(nu_value.into_value(span))
    }

    /// Try to get the cached JSON value from a NuNickelValue
    pub fn try_get_cached_json(
        plugin: &NickelPlugin,
        value: &Value,
    ) -> Result<Option<serde_json::Value>, LabeledError> {
        let custom_value = match value.as_custom_value() {
            Ok(custom_value) => custom_value,
            Err(_) => return Ok(None),
        };

        let nickel_custom_value = match custom_value
            .as_any()
            .downcast_ref::<NuNickelValueCustomValue>()
        {
            Some(value) => value,
            None => return Ok(None),
        };

        match plugin.cache.get(&nickel_custom_value.id) {
            Some(cached_value) => {
                if let Some(json) = cached_value.as_json() {
                    Ok(Some(json.clone()))
                } else {
                    Err(LabeledError::new("Type mismatch")
                        .with_label("Expected JSON value, found different type", value.span()))
                }
            }
            None => Err(LabeledError::new("Cached Nickel value not found")
                .with_label("This Nickel value is no longer available", value.span())),
        }
    }

    /// Try to get the cached source code from a NuNickelValue
    pub fn try_get_cached_source_code(
        plugin: &NickelPlugin,
        value: &Value,
    ) -> Result<Option<String>, LabeledError> {
        let custom_value = match value.as_custom_value() {
            Ok(custom_value) => custom_value,
            Err(_) => return Ok(None),
        };

        let nickel_custom_value = match custom_value
            .as_any()
            .downcast_ref::<NuNickelValueCustomValue>()
        {
            Some(value) => value,
            None => return Ok(None),
        };

        match plugin.cache.get(&nickel_custom_value.id) {
            Some(cached_value) => {
                if let Some(source) = cached_value.as_source_code() {
                    Ok(Some(source.clone()))
                } else {
                    Err(LabeledError::new("Type mismatch")
                        .with_label("Expected Nickel term with source code, found different type", value.span()))
                }
            }
            None => Err(LabeledError::new("Cached Nickel value not found")
                .with_label("This Nickel value is no longer available", value.span())),
        }
    }

    /// Try to get any cached value from a NuNickelValue
    pub fn try_get_cached_value(
        plugin: &NickelPlugin,
        value: &Value,
    ) -> Result<Option<CachedNickelValue>, LabeledError> {
        let custom_value = match value.as_custom_value() {
            Ok(custom_value) => custom_value,
            Err(_) => return Ok(None),
        };

        let nickel_custom_value = match custom_value
            .as_any()
            .downcast_ref::<NuNickelValueCustomValue>()
        {
            Some(value) => value,
            None => return Ok(None),
        };

        match plugin.cache.get(&nickel_custom_value.id) {
            Some(cached_value) => Ok(Some(cached_value)),
            None => Err(LabeledError::new("Cached Nickel value not found")
                .with_label("This Nickel value is no longer available", value.span())),
        }
    }
}