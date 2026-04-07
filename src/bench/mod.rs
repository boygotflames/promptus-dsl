pub mod tokenizer;

use anyhow::Result;

use crate::Document;
use crate::transpile::{self, Target};

use self::tokenizer::TokenCounter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchRow {
    pub name: &'static str,
    pub bytes: usize,
    pub tokens: usize,
    pub delta_bytes: i128,
    pub delta_tokens: i128,
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

        let mut lines = Vec::with_capacity(self.rows.len() + 1);
        lines.push(format!("tokenizer: {}", self.tokenizer));

        for row in &self.rows {
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

        lines.join("\n")
    }
}

pub fn measure_document(source: &str, document: &Document) -> Result<BenchReport> {
    let counter = TokenCounter::new()?;
    let representations = [
        ("source", source.to_owned()),
        ("plain", transpile::transpile(document, Target::Plain)),
        ("json-ir", transpile::transpile(document, Target::JsonIr)),
        ("shadow", transpile::transpile(document, Target::Shadow)),
    ];

    let baseline_bytes = representations
        .first()
        .map(|(_, text)| text.len())
        .unwrap_or_default();
    let baseline_tokens = representations
        .first()
        .map(|(_, text)| counter.count(text))
        .unwrap_or_default();

    let rows = representations
        .into_iter()
        .map(|(name, text)| {
            let bytes = text.len();
            let tokens = counter.count(&text);

            BenchRow {
                name,
                bytes,
                tokens,
                delta_bytes: signed_delta(bytes, baseline_bytes),
                delta_tokens: signed_delta(tokens, baseline_tokens),
            }
        })
        .collect();

    Ok(BenchReport {
        tokenizer: counter.encoding_name(),
        rows,
    })
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
