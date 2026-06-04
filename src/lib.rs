pub mod ast;
#[cfg(not(target_arch = "wasm32"))]
pub mod bench;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
pub mod composer;
pub mod diagnostics;
pub mod formatter;
pub mod include;
pub mod lexer;
pub mod lint;
pub mod merge;
pub mod parser;
pub mod provider;
pub mod transpile;
pub mod validator;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use ast::{Document, MappingEntry, Node, TopLevelKey};
pub use diagnostics::{Diagnostic, DiagnosticBag, DiagnosticPhase, Severity, Span};
pub use formatter::format_document;
pub use lint::{LintWarning, lint_document};
pub use parser::parse_str;
pub use provider::{Provider, ProviderProfile, ShadowProfile, SupportStatus, TokenizerProfile};
pub use validator::validate_document;
