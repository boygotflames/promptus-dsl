use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticPhase {
    Syntax,
    Semantic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub phase: DiagnosticPhase,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub code: Option<&'static str>,
}

impl Diagnostic {
    pub fn syntax_error<T: Into<String>>(message: T, span: Option<Span>) -> Self {
        Self {
            phase: DiagnosticPhase::Syntax,
            severity: Severity::Error,
            message: message.into(),
            span,
            code: None,
        }
    }

    pub fn semantic_error<T: Into<String>>(message: T, span: Option<Span>) -> Self {
        Self {
            phase: DiagnosticPhase::Semantic,
            severity: Severity::Error,
            message: message.into(),
            span,
            code: None,
        }
    }

    pub fn syntax_warning<T: Into<String>>(message: T, span: Option<Span>) -> Self {
        Self {
            phase: DiagnosticPhase::Syntax,
            severity: Severity::Warning,
            message: message.into(),
            span,
            code: None,
        }
    }

    pub fn semantic_warning<T: Into<String>>(message: T, span: Option<Span>) -> Self {
        Self {
            phase: DiagnosticPhase::Semantic,
            severity: Severity::Warning,
            message: message.into(),
            span,
            code: None,
        }
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match (self.phase, self.severity) {
            (DiagnosticPhase::Syntax, Severity::Error) => "syntax error",
            (DiagnosticPhase::Semantic, Severity::Error) => "semantic error",
            (DiagnosticPhase::Syntax, Severity::Warning) => "syntax warning",
            (DiagnosticPhase::Semantic, Severity::Warning) => "semantic warning",
        };

        match (self.span, &self.code) {
            (Some(span), Some(code)) => write!(
                f,
                "{label} at {}:{}: [{code}] {}",
                span.line, span.column, self.message
            ),
            (Some(span), None) => write!(
                f,
                "{label} at {}:{}: {}",
                span.line, span.column, self.message
            ),
            (None, Some(code)) => write!(f, "{label}: [{code}] {}", self.message),
            (None, None) => write!(f, "{label}: {}", self.message),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn syntax_error<T: Into<String>>(&mut self, message: T, span: Option<Span>) {
        self.push(Diagnostic::syntax_error(message, span));
    }

    pub fn semantic_error<T: Into<String>>(&mut self, message: T, span: Option<Span>) {
        self.push(Diagnostic::semantic_error(message, span));
    }

    pub fn syntax_warning<T: Into<String>>(&mut self, message: T, span: Option<Span>) {
        self.push(Diagnostic::syntax_warning(message, span));
    }

    pub fn semantic_warning<T: Into<String>>(&mut self, message: T, span: Option<Span>) {
        self.push(Diagnostic::semantic_warning(message, span));
    }

    pub fn extend(&mut self, other: Self) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

impl fmt::Display for DiagnosticBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{diagnostic}")?;
        }

        Ok(())
    }
}

impl Error for DiagnosticBag {}
