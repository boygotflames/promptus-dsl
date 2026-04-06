use crate::ast::Document;

pub mod json_ir;
pub mod plain;
pub mod shadow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Plain,
    Shadow,
    JsonIr,
}

pub fn transpile(document: &Document, target: Target) -> String {
    match target {
        Target::Plain => plain::transpile(document),
        Target::Shadow => shadow::transpile(document),
        Target::JsonIr => json_ir::transpile(document),
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
