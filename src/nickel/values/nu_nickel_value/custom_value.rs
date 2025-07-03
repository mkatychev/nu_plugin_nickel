use crate::nickel::values::NuNickelValue;
use crate::cache::CachedNickelValue;
use nu_protocol::{
    CustomValue, Record, ShellError, Span, Value,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuNickelValueCustomValue {
    pub id: Uuid,
    pub type_name: String,
    /// Optional cached data - skipped during serialization for thread safety
    #[serde(skip)]
    pub cached_value: Option<CachedNickelValue>,
}

impl NuNickelValueCustomValue {
    pub fn new(value: NuNickelValue) -> Self {
        Self {
            id: value.id,
            type_name: value.type_name,
            cached_value: None,
        }
    }

    /// Create a new custom value with optional cached data
    pub fn with_cached_value(value: NuNickelValue, cached_value: Option<CachedNickelValue>) -> Self {
        Self {
            id: value.id,
            type_name: value.type_name,
            cached_value,
        }
    }

    /// Get the cached value if available
    pub fn get_cached_value(&self) -> Option<&CachedNickelValue> {
        self.cached_value.as_ref()
    }
}

#[typetag::serde]
impl CustomValue for NuNickelValueCustomValue {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        self.type_name.clone()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        let mut record = Record::new();
        record.push("id", Value::string(self.id.to_string(), span));
        record.push("type", Value::string(&self.type_name, span));
        
        // Add cached value info if available
        if let Some(cached_value) = &self.cached_value {
            record.push("object_type", Value::string(cached_value.object_type(), span));
            record.push("reference_count", Value::int(cached_value.reference_count as i64, span));
            record.push("created", Value::string(cached_value.created.to_rfc3339(), span));
        } else {
            record.push("cached", Value::bool(false, span));
        }
        
        Ok(Value::record(record, span))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn notify_plugin_on_drop(&self) -> bool {
        true
    }

}