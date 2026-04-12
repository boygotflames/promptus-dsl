use anyhow::Result;

use crate::ast::Document;
use crate::provider::Provider;

pub mod json_ir;
pub mod plain;
pub mod shadow;
pub mod vars;

pub trait Emitter {
    fn emit(&self, document: &Document) -> String;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Plain,
    Shadow,
    JsonIr,
}

pub fn transpile(document: &Document, target: Target) -> String {
    transpile_with_provider(document, target, Provider::Generic)
        .expect("generic provider must support built-in transpilation targets")
}

pub fn transpile_with_provider(
    document: &Document,
    target: Target,
    provider: Provider,
) -> Result<String> {
    match target {
        Target::Plain => Ok(plain::PlainEmitter.emit(document)),
        Target::Shadow => shadow::emit_with_provider(document, provider),
        Target::JsonIr => Ok(json_ir::JsonIrEmitter.emit(document)),
    }
}

fn quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');

    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }

    quoted.push('"');
    quoted
}
