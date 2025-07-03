use super::*;
use crate::NickelPlugin;
use nu_plugin::Plugin;

#[test]
fn test_nickel_eval_simple_record() {
    let plugin = NickelPlugin::default();
    let eval_cmd = NickelEval;

    // Test parsing a simple record
    let source = "{ foo = 42, bar = \"hello\" }";
}
