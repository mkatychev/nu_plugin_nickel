use crate::nickel::values::NuNickelValue;
use nu_protocol::{
    CustomValue, IntoSpanned, LabeledError, PipelineData, Record, ShellError, Span, Spanned, Value,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuNickelValueCustomValue {
    pub id: Uuid,
    pub type_name: String,
}

impl NuNickelValueCustomValue {
    pub fn new(value: NuNickelValue) -> Self {
        Self {
            id: value.id,
            type_name: value.type_name,
        }
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
        Ok(Value::record(record, span))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn notify_plugin_on_drop(&self) -> bool {
        true
    }

    fn typetag_name(&self) -> &'static str {
        "NuNickelValueCustomValue"
    }

    fn typetag_deserialize(&self) {
        // Required by typetag
    }
}