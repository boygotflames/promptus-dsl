use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::bench::tokenizer::count_tokens;
use crate::parser::parse_str;
use crate::transpile::{self, Target};
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct BenchArgs {
    pub input: PathBuf,
}

pub fn run(args: BenchArgs) -> Result<()> {
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

    let plain = transpile::transpile(&document, Target::Plain);
    let shadow = transpile::transpile(&document, Target::Shadow);

    println!("source_tokens: {}", count_tokens(&source)?);
    println!("plain_tokens: {}", count_tokens(&plain)?);
    println!("shadow_tokens: {}", count_tokens(&shadow)?);

    Ok(())
}
