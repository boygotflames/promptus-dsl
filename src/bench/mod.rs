pub mod tokenizer;

use anyhow::Result;

use crate::Document;
use crate::transpile::{self, Target};

use self::tokenizer::TokenCounter;

pub const BASELINE_ROW_NAME: &str = "baseline";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchRow {
    pub name: &'static str,
    pub bytes: usize,
    pub tokens: usize,
    pub delta_bytes: i128,
    pub delta_tokens: i128,
    pub delta_bytes_vs_baseline: Option<i128>,
    pub delta_tokens_vs_baseline: Option<i128>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchReport {
    pub tokenizer: &'static str,
    pub rows: Vec<BenchRow>,
}

impl BenchReport {
    pub fn render(&self) -> String {
        let name_width = self
            .rows
            .iter()
            .map(|row| row.name.len())
            .max()
            .unwrap_or_default();
        let has_baseline = self
            .rows
            .iter()
            .any(|row| row.delta_bytes_vs_baseline.is_some());

        let mut lines = Vec::with_capacity(self.rows.len() + 1);
        lines.push(format!("tokenizer: {}", self.tokenizer));

        for row in &self.rows {
            if has_baseline {
                lines.push(format!(
                    "{:<width$} | bytes={} | tokens={} | delta_bytes={} | delta_tokens={} | delta_bytes_vs_baseline={} | delta_tokens_vs_baseline={}",
                    row.name,
                    row.bytes,
                    row.tokens,
                    format_signed(row.delta_bytes),
                    format_signed(row.delta_tokens),
                    format_signed(
                        row.delta_bytes_vs_baseline
                            .expect("baseline deltas must exist when baseline rendering is enabled")
                    ),
                    format_signed(
                        row.delta_tokens_vs_baseline
                            .expect("baseline deltas must exist when baseline rendering is enabled")
                    ),
                    width = name_width
                ));
            } else {
                lines.push(format!(
                    "{:<width$} | bytes={} | tokens={} | delta_bytes={} | delta_tokens={}",
                    row.name,
                    row.bytes,
                    row.tokens,
                    format_signed(row.delta_bytes),
                    format_signed(row.delta_tokens),
                    width = name_width
                ));
            }
        }

        lines.join("\n")
    }
}

pub fn measure_document(source: &str, document: &Document) -> Result<BenchReport> {
    measure_document_with_baseline(source, document, None)
}

pub fn measure_document_with_baseline(
    source: &str,
    document: &Document,
    baseline: Option<&str>,
) -> Result<BenchReport> {
    let counter = TokenCounter::new()?;
    let source_metrics = BenchMetrics::from_text(source, &counter);
    let baseline_metrics = baseline.map(|text| BenchMetrics::from_text(text, &counter));

    let mut representations = Vec::with_capacity(4 + usize::from(baseline.is_some()));
    representations.push(("source", source.to_owned()));

    if let Some(text) = baseline {
        representations.push((BASELINE_ROW_NAME, text.to_owned()));
    }

    representations.push(("plain", transpile::transpile(document, Target::Plain)));
    representations.push(("json-ir", transpile::transpile(document, Target::JsonIr)));
    representations.push(("shadow", transpile::transpile(document, Target::Shadow)));

    let rows = representations
        .into_iter()
        .map(|(name, text)| build_row(name, &text, &counter, source_metrics, baseline_metrics))
        .collect();

    Ok(BenchReport {
        tokenizer: counter.encoding_name(),
        rows,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BenchMetrics {
    bytes: usize,
    tokens: usize,
}

impl BenchMetrics {
    fn from_text(text: &str, counter: &TokenCounter) -> Self {
        Self {
            bytes: text.len(),
            tokens: counter.count(text),
        }
    }
}

fn build_row(
    name: &'static str,
    text: &str,
    counter: &TokenCounter,
    source_metrics: BenchMetrics,
    baseline_metrics: Option<BenchMetrics>,
) -> BenchRow {
    let metrics = BenchMetrics::from_text(text, counter);

    BenchRow {
        name,
        bytes: metrics.bytes,
        tokens: metrics.tokens,
        delta_bytes: signed_delta(metrics.bytes, source_metrics.bytes),
        delta_tokens: signed_delta(metrics.tokens, source_metrics.tokens),
        delta_bytes_vs_baseline: baseline_metrics
            .map(|baseline| signed_delta(metrics.bytes, baseline.bytes)),
        delta_tokens_vs_baseline: baseline_metrics
            .map(|baseline| signed_delta(metrics.tokens, baseline.tokens)),
    }
}

fn signed_delta(value: usize, baseline: usize) -> i128 {
    value as i128 - baseline as i128
}

fn format_signed(value: i128) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}
