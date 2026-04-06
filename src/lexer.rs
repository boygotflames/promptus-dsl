use crate::diagnostics::{DiagnosticBag, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineToken {
    pub line: usize,
    pub column: usize,
    pub indent: usize,
    pub content: String,
}

impl LineToken {
    pub fn span(&self) -> Span {
        Span::new(self.line, self.column)
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
            diagnostics.error(
                "tabs are not supported; use two-space indentation",
                Some(Span::new(line_number, 1)),
            );
            continue;
        }

        let indent = raw_line.chars().take_while(|ch| *ch == ' ').count();
        if indent % 2 != 0 {
            diagnostics.error(
                "indentation must use multiples of two spaces",
                Some(Span::new(line_number, 1)),
            );
            continue;
        }

        let content = raw_line[indent..].trim_end().to_owned();
        tokens.push(LineToken {
            line: line_number,
            column: indent + 1,
            indent,
            content,
        });
    }

    if diagnostics.has_errors() {
        Err(diagnostics)
    } else {
        Ok(tokens)
    }
}
