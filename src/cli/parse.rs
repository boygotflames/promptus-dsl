use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::parser::parse_str;

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: PathBuf,
}

pub fn run(args: ParseArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;

    let document = parse_str(&source).map_err(|diagnostics| {
        eprintln!("{diagnostics}");
        anyhow!("parse failed")
    })?;

    println!("{document:#?}");
    Ok(())
}
