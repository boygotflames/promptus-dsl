use std::collections::BTreeMap;

use crate::ast::{Document, Node, TopLevelKey};
use crate::diagnostics::{DiagnosticBag, Span};
use crate::lexer::{LineToken, tokenize_lines};

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
                ));
            }

            let (key_text, value, span) = self.parse_mapping_entry(0)?;
            let Some(key) = TopLevelKey::from_str(&key_text) else {
                return Err(single_error(
                    format!("unknown top-level key `{key_text}`"),
                    span,
                ));
            };

            if document.set(key, value).is_some() {
                return Err(single_error(
                    format!("duplicate top-level key `{}`", key.as_str()),
                    span,
                ));
            }
        }

        Ok(document)
    }

    fn parse_block(&mut self, expected_indent: usize) -> Result<Node, DiagnosticBag> {
        let Some(token) = self.current().cloned() else {
            return Err(single_error(
                "expected an indented block, but found end of input",
                Span::new(1, 1),
            ));
        };

        if token.indent != expected_indent {
            return Err(single_error(
                format!("expected indentation of {expected_indent} spaces"),
                token.span(),
            ));
        }

        if token.content.starts_with("- ") || token.content == "-" {
            self.parse_sequence(expected_indent).map(Node::Sequence)
        } else {
            self.parse_mapping(expected_indent).map(Node::Mapping)
        }
    }

    fn parse_sequence(&mut self, expected_indent: usize) -> Result<Vec<Node>, DiagnosticBag> {
        let mut items = Vec::new();

        while let Some(token) = self.current().cloned() {
            if token.indent < expected_indent {
                break;
            }

            if token.indent > expected_indent {
                return Err(single_error(
                    "unexpected indentation inside sequence",
                    token.span(),
                ));
            }

            if !token.content.starts_with('-') {
                break;
            }

            let remainder = token
                .content
                .strip_prefix('-')
                .unwrap_or_default()
                .trim_start();
            self.advance();

            if remainder.is_empty() {
                items.push(self.parse_nested_block(expected_indent, token.span())?);
            } else {
                items.push(Node::scalar(remainder));
            }
        }

        Ok(items)
    }

    fn parse_mapping(
        &mut self,
        expected_indent: usize,
    ) -> Result<BTreeMap<String, Node>, DiagnosticBag> {
        let mut mapping = BTreeMap::new();

        while let Some(token) = self.current().cloned() {
            if token.indent < expected_indent {
                break;
            }

            if token.indent > expected_indent {
                return Err(single_error(
                    "unexpected indentation inside mapping",
                    token.span(),
                ));
            }

            if token.content.starts_with('-') {
                return Err(single_error(
                    "list item found where a mapping entry was expected",
                    token.span(),
                ));
            }

            let (key, value, span) = self.parse_mapping_entry(expected_indent)?;
            if mapping.insert(key.clone(), value).is_some() {
                return Err(single_error(format!("duplicate key `{key}`"), span));
            }
        }

        Ok(mapping)
    }

    fn parse_mapping_entry(
        &mut self,
        expected_indent: usize,
    ) -> Result<(String, Node, Span), DiagnosticBag> {
        let token = self
            .current()
            .cloned()
            .ok_or_else(|| single_error("expected a mapping entry", Span::new(1, 1)))?;

        if token.indent != expected_indent {
            return Err(single_error(
                format!("expected indentation of {expected_indent} spaces"),
                token.span(),
            ));
        }

        let Some((key, inline_value)) = split_mapping_entry(&token.content) else {
            return Err(single_error(
                "expected `key: value` or `key:` syntax",
                token.span(),
            ));
        };

        self.advance();

        let value = match inline_value {
            Some(value) => Node::scalar(value),
            None => self.parse_nested_block(expected_indent, token.span())?,
        };

        Ok((key.to_owned(), value, token.span()))
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
            ));
        };

        let expected_indent = parent_indent + 2;
        if token.indent < expected_indent {
            return Err(single_error(
                "expected an indented block after `:`",
                parent_span,
            ));
        }

        if token.indent > expected_indent {
            return Err(single_error(
                format!("nested blocks must be indented by exactly {expected_indent} spaces"),
                token.span(),
            ));
        }

        self.parse_block(expected_indent)
    }

    fn current(&self) -> Option<&LineToken> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }
}

fn split_mapping_entry(content: &str) -> Option<(&str, Option<&str>)> {
    let (raw_key, raw_value) = content.split_once(':')?;
    let key = raw_key.trim();
    if key.is_empty() {
        return None;
    }

    let value = raw_value.trim();
    if value.is_empty() {
        Some((key, None))
    } else {
        Some((key, Some(value)))
    }
}

fn single_error<T: Into<String>>(message: T, span: Span) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::new();
    diagnostics.error(message, Some(span));
    diagnostics
}
