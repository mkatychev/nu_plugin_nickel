pub mod core;

use crate::NickelPlugin;
use nu_plugin::PluginCommand;

pub fn core_commands() -> Vec<Box<dyn PluginCommand<Plugin = NickelPlugin>>> {
    vec![
        Box::new(core::NickelEval),
        Box::new(core::NickelParse),
    ]
}