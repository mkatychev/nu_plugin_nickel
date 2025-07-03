use crate::NickelPlugin;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, Span, SyntaxShape, Type, Value,
};

#[derive(Clone)]
pub struct NickelEval;

impl PluginCommand for NickelEval {
    type Plugin = NickelPlugin;

    fn name(&self) -> &str {
        "nickel eval"
    }

    fn signature(&self) -> Signature {
        Signature::build("nickel eval")
            .input_output_types(vec![
                (Type::String, Type::Any),
                (Type::Nothing, Type::Any),
            ])
            .optional("path", SyntaxShape::Filepath, "Path to nickel file to evaluate")
            .switch("json", "Output as JSON", Some('j'))
            .switch("yaml", "Output as YAML", Some('y'))
            .switch("toml", "Output as TOML", Some('t'))
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Evaluate Nickel code and return the result"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Evaluate Nickel code from string",
                example: r#""{ foo = 42 }" | nickel eval"#,
                result: None,
            },
            Example {
                description: "Evaluate Nickel file",
                example: "nickel eval config.ncl",
                result: None,
            },
            Example {
                description: "Evaluate and output as JSON",
                example: r#""{ foo = 42 }" | nickel eval --json"#,
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &NickelPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let span = call.head;

        // Get the source code - either from input or from file
        let source = if let Some(path) = call.opt::<String>(0)? {
            // Read from file
            std::fs::read_to_string(&path).map_err(|e| {
                LabeledError::new(format!("Failed to read file: {}", e))
                    .with_label(format!("Cannot read file '{}'", path), span)
            })?
        } else {
            // Read from input
            match input {
                PipelineData::Value(Value::String { val, .. }, _) => val,
                PipelineData::Empty => {
                    return Err(LabeledError::new("No input provided")
                        .with_label("Provide Nickel code as input or specify a file path", span))
                }
                _ => {
                    return Err(LabeledError::new("Invalid input type")
                        .with_label("Expected string input", span))
                }
            }
        };

        // For now, return a simple evaluation result
        // TODO: Implement actual Nickel evaluation
        let result = if call.has_flag("json")? {
            Value::string(format!(r#"{{"nickel_source": "{}"}}"#, source), span)
        } else if call.has_flag("yaml")? {
            Value::string(format!("nickel_source: {}", source), span)
        } else if call.has_flag("toml")? {
            Value::string(format!("nickel_source = \"{}\"", source), span)
        } else {
            // Return as a record for now
            let mut record = Record::new();
            record.push("nickel_source", Value::string(source, span));
            record.push("status", Value::string("parsed", span));
            Value::record(record, span)
        };

        Ok(PipelineData::Value(result, None))
    }
}