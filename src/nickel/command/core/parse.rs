use crate::nickel::values::NuNickelValue;
use crate::NickelPlugin;
use nickel_lang_core::{
    cache::resolvers::DummyResolver,
    program::Program,
};
use nu_plugin::{EngineInterface, EvaluatedCall, LabeledError, PluginCommand};
use nu_protocol::{
    Category, Example, PipelineData, Record, Signature, Span, SyntaxShape, Type, Value,
};
use std::io::Cursor;

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
                (Type::String, Type::Custom("NickelValue".to_string())),
                (Type::Nothing, Type::Custom("NickelValue".to_string())),
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
        engine: &EngineInterface,
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

        // Parse the code
        let result = self.parse_nickel_code(plugin, engine, &source, span)?;

        Ok(PipelineData::Value(result, None))
    }
}

impl NickelParse {
    fn parse_nickel_code(
        &self,
        plugin: &NickelPlugin,
        engine: &EngineInterface,
        source: &str,
        span: Span,
    ) -> Result<Value, LabeledError> {
        // Create a program from the source
        let mut program = Program::<DummyResolver>::new_from_source(
            Cursor::new(source),
            "<input>".to_string(),
            std::io::stderr(),
        )
        .map_err(|e| {
            LabeledError::new(format!("Failed to create program: {}", e))
                .with_label("Invalid Nickel code", span)
        })?;

        // Parse the program
        program.parse().map_err(|e| {
            LabeledError::new(format!("Parse error: {}", e))
                .with_label("Failed to parse Nickel code", span)
        })?;

        // Get the parsed term
        let term = program.into_inner();

        // Cache the parsed term and return a Nickel value
        NuNickelValue::cache_and_to_value(plugin, engine, term, span)
    }
}