use crate::nickel::values::NuNickelValue;
use crate::NickelPlugin;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};

#[derive(Clone)]
pub struct NickelParse;

impl PluginCommand for NickelParse {
    type Plugin = NickelPlugin;

    fn name(&self) -> &str {
        "nickel parse"
    }

    fn signature(&self) -> Signature {
        Signature::build("nickel parse")
            .input_output_types(vec![
                (Type::String, Type::Custom("NickelValue".to_string().into())),
                (Type::Nothing, Type::Custom("NickelValue".to_string().into())),
            ])
            .optional("path", SyntaxShape::Filepath, "Path to nickel file to parse")
            .category(Category::Conversions)
    }

    fn description(&self) -> &str {
        "Parse Nickel code and return the AST as a Nickel value"
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Parse Nickel code from string",
                example: r#""{ foo = 42 }" | nickel parse"#,
                result: None,
            },
            Example {
                description: "Parse Nickel file",
                example: "nickel parse config.ncl",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        plugin: &NickelPlugin,
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

        // For now, create a simple JSON representation of the parse
        let json_value = serde_json::json!({
            "source": source,
            "ast": "placeholder_ast",
            "status": "parsed"
        });

        let result = NuNickelValue::cache_json_value(plugin, json_value, span)?;

        Ok(PipelineData::Value(result, None))
    }
}