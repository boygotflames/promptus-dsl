use crate::ast::Document;

pub mod json_ir;
pub mod plain;
pub mod shadow;

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
    match target {
        Target::Plain => plain::PlainEmitter.emit(document),
        Target::Shadow => shadow::ShadowEmitter.emit(document),
        Target::JsonIr => json_ir::JsonIrEmitter.emit(document),
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

fn format_plain_scalar(value: &str) -> String {
    if is_plain_bare_scalar(value) {
        value.to_owned()
    } else {
        quote(value)
    }
}

fn is_plain_bare_scalar(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}
