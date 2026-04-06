use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Args, ValueEnum};

use crate::parser::parse_str;
use crate::transpile::{self, Target};
use crate::validator::validate_document;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TargetArg {
    Plain,
    Shadow,
    JsonIr,
}

impl From<TargetArg> for Target {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::Plain => Target::Plain,
            TargetArg::Shadow => Target::Shadow,
            TargetArg::JsonIr => Target::JsonIr,
        }
    }
}

#[derive(Debug, Args)]
pub struct TranspileArgs {
    pub input: PathBuf,

    #[arg(long, value_enum, default_value_t = TargetArg::Plain)]
    pub target: TargetArg,
}

pub fn run(args: TranspileArgs) -> Result<()> {
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

    let rendered = transpile::transpile(&document, args.target.into());
    println!("{rendered}");
    Ok(())
}
