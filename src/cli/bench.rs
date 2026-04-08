use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::bench;
use crate::parser::parse_str;
use crate::provider::Provider;
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct BenchArgs {
    pub input: PathBuf,

    #[arg(long, value_enum, default_value_t = Provider::Generic)]
    pub provider: Provider,

    #[arg(long)]
    pub baseline: Option<PathBuf>,
}

pub fn run(args: BenchArgs) -> Result<()> {
    let report = execute(args)?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(report.as_bytes())
        .context("failed to write bench report to stdout")?;
    stdout
        .write_all(b"\n")
        .context("failed to finalize bench report output")?;
    Ok(())
}

pub fn execute(args: BenchArgs) -> Result<String> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;

    let document = parse_str(&source).map_err(|diagnostics| {
        eprintln!("{diagnostics}");
        anyhow!("parse failed")
    })?;

    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        eprintln!("{diagnostics}");
        return Err(anyhow!("validation failed"));
    }

    let baseline = match args.baseline {
        Some(path) => Some(
            fs::read_to_string(&path)
                .with_context(|| format!("failed to read baseline {}", path.display()))?,
        ),
        None => None,
    };

    let report = bench::measure_document_with_provider_and_baseline(
        &source,
        &document,
        args.provider,
        baseline.as_deref(),
    )?;
    Ok(report.render())
}
