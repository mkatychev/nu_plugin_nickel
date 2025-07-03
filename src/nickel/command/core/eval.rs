use crate::nickel::values::NuNickelValue;
use crate::NickelPlugin;
use nickel_lang_core::{
    cache::resolvers::DummyResolver,
    eval::{cache::lazy::LazyEvalCache, VirtualMachine},
    parser::lexer::Lexer,
    parser::{self, ErrorTolerantParser},
    program::Program,
    serialize,
    term::{RuntimeContract, StrChunk},
};
use nu_plugin::{EngineInterface, EvaluatedCall, LabeledError, PluginCommand};
use nu_protocol::{
    Category, Example, PipelineData, Record, ShellError, Signature, Span, SyntaxShape, Type, Value,
};
use std::io::Cursor;

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

        // Parse and evaluate
        let result = self.eval_nickel_code(&source, call, span)?;

        Ok(PipelineData::Value(result, None))
    }
}

impl NickelEval {
    fn eval_nickel_code(
        &self,
        source: &str,
        call: &EvaluatedCall,
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

        // Type check if needed
        program.typecheck().map_err(|e| {
            LabeledError::new(format!("Type error: {}", e))
                .with_label("Type checking failed", span)
        })?;

        // Evaluate the program
        let evaluated = program.eval_full().map_err(|e| {
            LabeledError::new(format!("Evaluation error: {}", e))
                .with_label("Failed to evaluate Nickel code", span)
        })?;

        // Convert to appropriate output format
        if call.has_flag("json")? {
            let json_str = serialize::to_json(&evaluated).map_err(|e| {
                LabeledError::new(format!("JSON serialization error: {}", e))
                    .with_label("Cannot convert to JSON", span)
            })?;
            Ok(Value::string(json_str, span))
        } else if call.has_flag("yaml")? {
            let yaml_str = serialize::to_yaml(&evaluated).map_err(|e| {
                LabeledError::new(format!("YAML serialization error: {}", e))
                    .with_label("Cannot convert to YAML", span)
            })?;
            Ok(Value::string(yaml_str, span))
        } else if call.has_flag("toml")? {
            let toml_str = serialize::to_toml(&evaluated).map_err(|e| {
                LabeledError::new(format!("TOML serialization error: {}", e))
                    .with_label("Cannot convert to TOML", span)
            })?;
            Ok(Value::string(toml_str, span))
        } else {
            // Convert to Nushell value
            self.nickel_to_nu_value(&evaluated, span)
        }
    }

    fn nickel_to_nu_value(
        &self,
        value: &nickel_lang_core::term::RichTerm,
        span: Span,
    ) -> Result<Value, LabeledError> {
        use nickel_lang_core::term::Term;

        match value.as_ref() {
            Term::Null => Ok(Value::nothing(span)),
            Term::Bool(b) => Ok(Value::bool(*b, span)),
            Term::Num(n) => Ok(Value::float(n.into(), span)),
            Term::Str(s) => Ok(Value::string(s.clone(), span)),
            Term::Array(arr, _) => {
                let mut values = Vec::new();
                for item in arr.iter() {
                    values.push(self.nickel_to_nu_value(item, span)?);
                }
                Ok(Value::list(values, span))
            }
            Term::Record(record) => {
                let mut nu_record = Record::new();
                for (key, value) in record.fields.iter() {
                    if let Some(term) = &value.value {
                        let nu_value = self.nickel_to_nu_value(term, span)?;
                        nu_record.push(key.ident().to_string(), nu_value);
                    }
                }
                Ok(Value::record(nu_record, span))
            }
            _ => {
                // For other types, convert to string representation
                Ok(Value::string(format!("{:?}", value), span))
            }
        }
    }
}