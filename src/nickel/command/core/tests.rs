use super::*;
use crate::NickelPlugin;
use nu_plugin_test_support::PluginTest;
use nu_protocol::{PipelineData, Span, Value};

#[test]
fn test_nickel_eval_record_contract() {
    let plugin = NickelPlugin::default();
    let mut plugin_test = PluginTest::new("nickel", Box::new(plugin.clone()));

    // Test the record-contract.ncl example
    let result = plugin_test
        .eval(&format!(
            r#"open "{}/examples/record-contract.ncl" | get kind"#,
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("Failed to evaluate record-contract.ncl");

    // Should successfully pipe kind from the evaluation
    assert!(result.into_value(Span::test_data()).is_ok());
}

#[test]
fn test_nickel_eval_cargo_error() {
    let plugin = NickelPlugin::default();
    let mut plugin_test = PluginTest::new("nickel", Box::new(plugin.clone()));

    // Test the cargo-eval.ncl example - this should error
    let result = plugin_test.eval(&format!(
        r#"open "{}/examples/cargo-eval.ncl""#,
        env!("CARGO_MANIFEST_DIR")
    ));

    // Should error due to missing contracts.ncl and workspace-crate/Cargo.toml
    assert!(result.is_err());
}

#[test]
fn test_nickel_eval_simple_record() {
    let plugin = NickelPlugin::default();
    let mut plugin_test = PluginTest::new("nickel", Box::new(plugin.clone()));

    let result = plugin_test
        .eval(r#""{ foo = 42, bar = \"hello\" }" | nickel eval"#)
        .expect("Failed to evaluate simple record");

    let value = result.into_value(Span::test_data()).expect("Failed to get value");

    // Should return a record with foo and bar fields
    if let Value::Record { val, .. } = value {
        assert!(val.contains("foo"));
        assert!(val.contains("bar"));
    } else {
        panic!("Expected record, got: {:?}", value);
    }
}

#[test]
fn test_nickel_eval_json_output() {
    let plugin = NickelPlugin::default();
    let mut plugin_test = PluginTest::new("nickel", Box::new(plugin.clone()));

    let result = plugin_test
        .eval(r#""{ foo = 42 }" | nickel eval --json"#)
        .expect("Failed to evaluate with JSON output");

    let value = result.into_value(Span::test_data()).expect("Failed to get value");

    // Should return a JSON string
    if let Value::String { val, .. } = value {
        assert!(val.contains("foo"));
        assert!(val.contains("42"));
    } else {
        panic!("Expected string, got: {:?}", value);
    }
}