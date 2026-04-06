use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::parser::parse_str;
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct ValidateArgs {
    pub input: PathBuf,
}

pub fn run(args: ValidateArgs) -> Result<()> {
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

    println!("validation passed");
    Ok(())
}
