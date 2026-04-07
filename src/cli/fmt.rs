use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::formatter;
use crate::parser::parse_str;
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct FmtArgs {
    pub input: PathBuf,

    #[arg(long)]
    pub write: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDestination {
    Stdout,
    Write(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FmtExecution {
    pub rendered: String,
    pub destination: OutputDestination,
}

pub fn run(args: FmtArgs) -> Result<()> {
    let execution = execute(args)?;

    if execution.destination == OutputDestination::Stdout {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(execution.rendered.as_bytes())
            .context("failed to write formatted output to stdout")?;
    }

    Ok(())
}

pub fn execute(args: FmtArgs) -> Result<FmtExecution> {
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

    let rendered = formatter::format_document(&document);
    let destination = if args.write {
        fs::write(&args.input, &rendered)
            .with_context(|| format!("failed to write {}", args.input.display()))?;
        OutputDestination::Write(args.input)
    } else {
        OutputDestination::Stdout
    };

    Ok(FmtExecution {
        rendered,
        destination,
    })
}
