use crate::ast::Document;
use crate::formatter::format_document;

use super::Emitter;
use super::vars;

pub struct PlainEmitter;

impl Emitter for PlainEmitter {
    fn emit(&self, document: &Document) -> String {
        // Expand {var_name} references before formatting.
        // The formatter preserves {var} verbatim; plain output shows expanded values.
        let expanded = vars::expand_document(document);
        format_document(&expanded)
    }
}
