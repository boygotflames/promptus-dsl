//! WebAssembly bindings for the llm_format core.
//!
//! Exposes parse, validate, transpile, and lint
//! as JavaScript-callable functions.

use wasm_bindgen::prelude::*;

use crate::lint::lint_document;
use crate::parser::parse_str;
use crate::provider::Provider;
use crate::transpile::{transpile_with_provider, Target};
use crate::validator::validate_document;

/// Set up better panic messages in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Compile a .llm source string to the requested target.
///
/// target: "plain" | "json-ir" | "shadow" |
///         "openai-chat" | "anthropic-messages"
///
/// Returns a JSON object:
/// { ok: true, output: "..." }
/// or
/// { ok: false, errors: ["..."] }
#[wasm_bindgen]
pub fn compile(source: &str, target: &str) -> String {
    let document = match parse_str(source) {
        Ok(doc) => doc,
        Err(bag) => {
            let errors: Vec<String> = bag.iter().map(|d| d.to_string()).collect();
            return serde_json::json!({
                "ok": false,
                "errors": errors
            })
            .to_string();
        }
    };

    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        let errors: Vec<String> = diagnostics.iter().map(|d| d.to_string()).collect();
        return serde_json::json!({
            "ok": false,
            "errors": errors
        })
        .to_string();
    }

    let transpile_target = match target {
        "plain" => Target::Plain,
        "json-ir" => Target::JsonIr,
        "shadow" => Target::Shadow,
        "openai-chat" => Target::OpenAiChat,
        "anthropic-messages" => Target::AnthropicMessages,
        other => {
            return serde_json::json!({
                "ok": false,
                "errors": [format!("unknown target: {}", other)]
            })
            .to_string();
        }
    };

    match transpile_with_provider(&document, transpile_target, Provider::Generic) {
        Ok(output) => serde_json::json!({
            "ok": true,
            "output": output
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "errors": [e.to_string()]
        })
        .to_string(),
    }
}

/// Validate a .llm source string.
///
/// Returns:
/// { ok: true }
/// or
/// { ok: false, errors: ["[E101] missing required key: `agent`", ...] }
#[wasm_bindgen]
pub fn validate(source: &str) -> String {
    let document = match parse_str(source) {
        Ok(doc) => doc,
        Err(bag) => {
            let errors: Vec<String> = bag.iter().map(|d| d.to_string()).collect();
            return serde_json::json!({
                "ok": false,
                "errors": errors
            })
            .to_string();
        }
    };

    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        let errors: Vec<String> = diagnostics.iter().map(|d| d.to_string()).collect();
        serde_json::json!({
            "ok": false,
            "errors": errors
        })
        .to_string()
    } else {
        serde_json::json!({ "ok": true }).to_string()
    }
}

/// Lint a .llm source string.
///
/// Returns:
/// { warnings: [{ code: "L001", message: "..." }, ...] }
#[wasm_bindgen]
pub fn lint(source: &str) -> String {
    let document = match parse_str(source) {
        Ok(doc) => doc,
        Err(_) => {
            return serde_json::json!({
                "warnings": []
            })
            .to_string();
        }
    };

    let warnings = lint_document(&document);
    let w: Vec<serde_json::Value> = warnings
        .iter()
        .map(|w| {
            serde_json::json!({
                "code": w.code,
                "message": w.message
            })
        })
        .collect();

    serde_json::json!({ "warnings": w }).to_string()
}
