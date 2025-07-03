pub mod custom_value;

use crate::cache::CachedItem;
use crate::NickelPlugin;
use nu_plugin::EngineInterface;
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

    pub fn cache_and_to_value(
        plugin: &NickelPlugin,
        engine: &EngineInterface,
        value: nickel_lang_core::term::RichTerm,
        span: Span,
    ) -> Result<Value, LabeledError> {
        let id = plugin.cache.insert_value(value);
        let nu_value = NuNickelValue::new(id, "NickelValue".to_string());
        Ok(nu_value.into_value(span))
    }

    pub fn try_from_value(
        plugin: &NickelPlugin,
        value: &Value,
    ) -> Result<Option<nickel_lang_core::term::RichTerm>, LabeledError> {
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
            Some(CachedItem::Value(value)) => Ok(Some(value)),
            Some(CachedItem::Term(_)) => Ok(None),
            None => Err(LabeledError::new("Cached Nickel value not found")
                .with_label("This Nickel value is no longer available", value.span())),
        }
    }
}