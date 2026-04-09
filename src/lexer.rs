use crate::diagnostics::{Diagnostic, DiagnosticBag, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentifierToken {
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScalarToken {
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingLine {
    pub key: IdentifierToken,
    pub colon: Span,
    pub value: Option<ScalarToken>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItemLine {
    pub dash: Span,
    pub value: Option<ScalarToken>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineKind {
    Mapping(MappingLine),
    ListItem(ListItemLine),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineToken {
    pub indent: usize,
    pub kind: LineKind,
}

impl LineToken {
    pub fn span(&self) -> Span {
        match &self.kind {
            LineKind::Mapping(mapping) => mapping.key.span,
            LineKind::ListItem(item) => item.dash,
        }
    }
}

pub fn tokenize_lines(source: &str) -> Result<Vec<LineToken>, DiagnosticBag> {
    let mut diagnostics = DiagnosticBag::new();
    let mut tokens = Vec::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if raw_line.contains('\t') {
            diagnostics.push(
                Diagnostic::syntax_error(
                    "tabs are not supported; use two-space indentation",
                    Some(Span::new(line_number, 1)),
                )
                .with_code("E001"),
            );
            continue;
        }

        let indent = raw_line.chars().take_while(|ch| *ch == ' ').count();
        if indent % 2 != 0 {
            diagnostics.push(
                Diagnostic::syntax_error(
                    "indentation must use multiples of two spaces",
                    Some(Span::new(line_number, 1)),
                )
                .with_code("E002"),
            );
            continue;
        }

        let content = raw_line[indent..].trim_end();
        match tokenize_content(content, line_number, indent + 1) {
            Ok(kind) => tokens.push(LineToken { indent, kind }),
            Err(line_diagnostics) => diagnostics.extend(line_diagnostics),
        }
    }

    if diagnostics.has_errors() {
        Err(diagnostics)
    } else {
        Ok(tokens)
    }
}

fn tokenize_content(content: &str, line: usize, column: usize) -> Result<LineKind, DiagnosticBag> {
    if let Some(remainder) = content.strip_prefix('-') {
        tokenize_list_item(remainder, line, column)
    } else {
        tokenize_mapping(content, line, column)
    }
}

fn tokenize_list_item(
    remainder: &str,
    line: usize,
    column: usize,
) -> Result<LineKind, DiagnosticBag> {
    let dash = Span::new(line, column);

    if !remainder.is_empty() && !remainder.starts_with(' ') {
        return Err(single_error(
            "expected a space after `-` in a list item",
            dash,
            "E003",
        ));
    }

    let value_text = remainder.trim_start();
    let value = if value_text.is_empty() {
        None
    } else {
        Some(parse_scalar(
            value_text,
            Span::new(line, column + (remainder.len() - value_text.len()) + 1),
        )?)
    };

    Ok(LineKind::ListItem(ListItemLine { dash, value }))
}

fn tokenize_mapping(content: &str, line: usize, column: usize) -> Result<LineKind, DiagnosticBag> {
    let mut chars = content.char_indices();
    let Some((_, first)) = chars.next() else {
        return Err(single_error(
            "expected a mapping entry or list item",
            Span::new(line, column),
            "E004",
        ));
    };

    if !is_identifier_start(first) {
        return Err(single_error(
            "expected an identifier at the start of a mapping entry",
            Span::new(line, column),
            "E005",
        ));
    }

    let mut key_end = first.len_utf8();
    for (index, ch) in chars {
        if is_identifier_continue(ch) {
            key_end = index + ch.len_utf8();
            continue;
        }
        break;
    }

    let key = &content[..key_end];
    let after_key = &content[key_end..];
    let whitespace_len = after_key
        .chars()
        .take_while(|ch| ch.is_ascii_whitespace())
        .count();
    let after_whitespace = &after_key[whitespace_len..];

    let Some(after_colon) = after_whitespace.strip_prefix(':') else {
        return Err(single_error(
            "expected `:` after mapping key",
            Span::new(line, column + key_end),
            "E006",
        ));
    };

    let colon_column = column + key_end + whitespace_len;
    let value_text = after_colon.trim_start();
    let value = if value_text.is_empty() {
        None
    } else {
        Some(parse_scalar(
            value_text,
            Span::new(
                line,
                colon_column + 1 + (after_colon.len() - value_text.len()),
            ),
        )?)
    };

    Ok(LineKind::Mapping(MappingLine {
        key: IdentifierToken {
            value: key.to_owned(),
            span: Span::new(line, column),
        },
        colon: Span::new(line, colon_column),
        value,
    }))
}

fn parse_scalar(source: &str, span: Span) -> Result<ScalarToken, DiagnosticBag> {
    let mut chars = source.chars();
    match chars.next() {
        Some('"') | Some('\'') => parse_quoted_scalar(source, span),
        Some(_) => Ok(ScalarToken {
            value: source.to_owned(),
            span,
        }),
        None => Err(single_error("expected a scalar value", span, "E007")),
    }
}

fn parse_quoted_scalar(source: &str, span: Span) -> Result<ScalarToken, DiagnosticBag> {
    let quote = source
        .chars()
        .next()
        .expect("quoted scalar must have an opening quote");
    let mut chars = source.char_indices().skip(1);
    let mut value = String::new();

    while let Some((index, ch)) = chars.next() {
        if ch == quote {
            let trailing = source[index + ch.len_utf8()..].trim();
            if !trailing.is_empty() {
                return Err(single_error(
                    "unexpected trailing characters after quoted scalar",
                    Span::new(span.line, span.column + index + ch.len_utf8()),
                    "E008",
                ));
            }

            return Ok(ScalarToken { value, span });
        }

        if ch == '\\' {
            let Some((escape_index, escaped)) = chars.next() else {
                return Err(single_error(
                    "unterminated escape sequence in quoted scalar",
                    Span::new(span.line, span.column + index),
                    "E009",
                ));
            };

            value.push(match escaped {
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => {
                    return Err(single_error(
                        format!("unsupported escape sequence `\\{escaped}`"),
                        Span::new(span.line, span.column + escape_index),
                        "E010",
                    ));
                }
            });
            continue;
        }

        value.push(ch);
    }

    Err(single_error("unterminated quoted scalar", span, "E011"))
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_ascii_alphanumeric()
}

fn single_error<T: Into<String>>(message: T, span: Span, code: &'static str) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::new();
    diagnostics.push(Diagnostic::syntax_error(message, Some(span)).with_code(code));
    diagnostics
}
