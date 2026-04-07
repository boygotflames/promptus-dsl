use crate::ast::Document;
use crate::formatter::format_document;

use super::Emitter;

pub struct PlainEmitter;

impl Emitter for PlainEmitter {
    fn emit(&self, document: &Document) -> String {
        format_document(document)
    }
}
