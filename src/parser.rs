use crate::ast::{Document, MappingEntry, Node, TopLevelKey};
use crate::diagnostics::{Diagnostic, DiagnosticBag, Span};
use crate::lexer::{LineKind, LineToken, tokenize_lines};

pub fn parse_str(source: &str) -> Result<Document, DiagnosticBag> {
    let tokens = tokenize_lines(source)?;
    Parser::new(tokens).parse_document()
}

struct Parser {
    tokens: Vec<LineToken>,
    cursor: usize,
}

impl Parser {
    fn new(tokens: Vec<LineToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn parse_document(mut self) -> Result<Document, DiagnosticBag> {
        let mut document = Document::default();

        while let Some(token) = self.current().cloned() {
            if token.indent != 0 {
                return Err(single_error(
                    "top-level keys must start at column 1",
                    token.span(),
                    "E012",
                ));
            }

            let (key, value, span) = self.parse_top_level_entry()?;
            let Some(key) = TopLevelKey::from_keyword(&key) else {
                return Err(single_error(
                    format!("unknown top-level key `{key}`"),
                    span,
                    "E013",
                ));
            };

            if document.set(key, value).is_some() {
                return Err(single_error(
                    format!("duplicate top-level key `{}`", key.as_str()),
                    span,
                    "E014",
                ));
            }
        }

        Ok(document)
    }

    fn parse_top_level_entry(&mut self) -> Result<(String, Node, Span), DiagnosticBag> {
        let token = self
            .current()
            .cloned()
            .ok_or_else(|| single_error("expected a top-level entry", Span::new(1, 1), "E015"))?;
        let span = token.span();

        match token.kind {
            LineKind::Mapping(mapping) => {
                self.advance();
                let value = match mapping.value {
                    Some(value) => Node::scalar_at(value.value, mapping.key.span),
                    None => self.parse_nested_block(0, mapping.key.span)?,
                };

                Ok((mapping.key.value, value, mapping.key.span))
            }
            LineKind::ListItem(_) => Err(single_error(
                "top-level entries must be mappings, not list items",
                span,
                "E016",
            )),
        }
    }

    fn parse_block(&mut self, expected_indent: usize, span: Span) -> Result<Node, DiagnosticBag> {
        let Some(token) = self.current().cloned() else {
            return Err(single_error(
                "expected an indented block, but found end of input",
                Span::new(1, 1),
                "E017",
            ));
        };

        if token.indent != expected_indent {
            return Err(single_error(
                format!("expected indentation of {expected_indent} spaces"),
                token.span(),
                "E018",
            ));
        }

        match token.kind {
            LineKind::ListItem(_) => self.parse_sequence(expected_indent, span),
            LineKind::Mapping(_) => self.parse_mapping(expected_indent, span),
        }
    }

    fn parse_sequence(
        &mut self,
        expected_indent: usize,
        span: Span,
    ) -> Result<Node, DiagnosticBag> {
        let mut items = Vec::new();

        while let Some(token) = self.current().cloned() {
            if token.indent < expected_indent {
                break;
            }

            if token.indent > expected_indent {
                return Err(single_error(
                    "unexpected indentation inside sequence",
                    token.span(),
                    "E019",
                ));
            }

            let LineKind::ListItem(item) = token.kind else {
                break;
            };

            self.advance();

            if let Some(value) = item.value {
                items.push(Node::scalar_at(value.value, item.dash));
            } else {
                items.push(self.parse_nested_block(expected_indent, item.dash)?);
            }
        }

        Ok(Node::sequence_at(items, span))
    }

    fn parse_mapping(&mut self, expected_indent: usize, span: Span) -> Result<Node, DiagnosticBag> {
        let mut entries = Vec::new();

        while let Some(token) = self.current().cloned() {
            if token.indent < expected_indent {
                break;
            }

            if token.indent > expected_indent {
                return Err(single_error(
                    "unexpected indentation inside mapping",
                    token.span(),
                    "E018",
                ));
            }

            if matches!(&token.kind, LineKind::ListItem(_)) {
                return Err(single_error(
                    "list item found where a mapping entry was expected",
                    token.span(),
                    "E020",
                ));
            }

            let (key, value, entry_span) = self.parse_mapping_entry(expected_indent)?;
            entries.push(MappingEntry::new(key, value, entry_span));
        }

        Ok(Node::mapping_at(entries, span))
    }

    fn parse_mapping_entry(
        &mut self,
        expected_indent: usize,
    ) -> Result<(String, Node, Span), DiagnosticBag> {
        let token = self
            .current()
            .cloned()
            .ok_or_else(|| single_error("expected a mapping entry", Span::new(1, 1), "E021"))?;

        if token.indent != expected_indent {
            return Err(single_error(
                format!("expected indentation of {expected_indent} spaces"),
                token.span(),
                "E018",
            ));
        }
        let span = token.span();

        let LineKind::Mapping(mapping) = token.kind else {
            return Err(single_error(
                "expected a mapping entry, but found a list item",
                span,
                "E022",
            ));
        };

        self.advance();

        let value = match mapping.value {
            Some(value) => Node::scalar_at(value.value, mapping.key.span),
            None => self.parse_nested_block(expected_indent, mapping.key.span)?,
        };

        Ok((mapping.key.value, value, mapping.key.span))
    }

    fn parse_nested_block(
        &mut self,
        parent_indent: usize,
        parent_span: Span,
    ) -> Result<Node, DiagnosticBag> {
        let Some(token) = self.current().cloned() else {
            return Err(single_error(
                "expected an indented block after `:`",
                parent_span,
                "E023",
            ));
        };

        let expected_indent = parent_indent + 2;
        if token.indent < expected_indent {
            return Err(single_error(
                "expected an indented block after `:`",
                parent_span,
                "E023",
            ));
        }

        if token.indent > expected_indent {
            return Err(single_error(
                format!("nested blocks must be indented by exactly {expected_indent} spaces"),
                token.span(),
                "E024",
            ));
        }

        self.parse_block(expected_indent, parent_span)
    }

    fn current(&self) -> Option<&LineToken> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }
}

fn single_error<T: Into<String>>(message: T, span: Span, code: &'static str) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::new();
    diagnostics.push(Diagnostic::syntax_error(message, Some(span)).with_code(code));
    diagnostics
}
