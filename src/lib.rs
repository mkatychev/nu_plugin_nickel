use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, PluginCommand, LabeledError};
use nu_protocol::{CustomValue, PipelineData, Signature, Span, Value};

mod cache;
mod nickel;

use cache::NickelCache;
use nickel::command;

pub struct NickelPlugin {
    pub cache: NickelCache,
}

impl Default for NickelPlugin {
    fn default() -> Self {
        Self {
            cache: NickelCache::default(),
        }
    }
}

impl Plugin for NickelPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        command::core_commands()
    }

    fn custom_value_dropped(
        &self,
        _engine: &nu_plugin::EngineInterface,
        custom_value: Box<dyn CustomValue>,
    ) -> Result<(), LabeledError> {
        let custom_value = custom_value
            .as_any()
            .downcast_ref::<nickel::values::NuNickelValueCustomValue>();

        if let Some(custom_value) = custom_value {
            self.cache.remove(&custom_value.id);
        }

        Ok(())
    }
}

pub fn serve() {
    serve_plugin(&NickelPlugin::default(), MsgPackSerializer {})
}