pub mod ast;
pub mod bench;
pub mod cli;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod transpile;
pub mod validator;

pub use ast::{Document, Node, TopLevelKey};
pub use diagnostics::{Diagnostic, DiagnosticBag, Severity, Span};
pub use parser::parse_str;
pub use validator::validate_document;
