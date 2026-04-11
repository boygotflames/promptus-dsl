use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::diagnostics::Severity;
use crate::parser::parse_str;
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct ValidateArgs {
    pub input: PathBuf,
}

pub fn run(args: ValidateArgs) -> Result<()> {
    let source = match fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to read {}: {e}", args.input.display());
            std::process::exit(2);
        }
    };

    let document = match parse_str(&source) {
        Ok(d) => d,
        Err(diagnostics) => {
            let error_count = diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let mut stderr = io::stderr().lock();
            writeln!(stderr, "{diagnostics}").ok();
            writeln!(
                stderr,
                "✗ invalid  {}  ({error_count} error(s))",
                args.input.display()
            )
            .ok();
            std::process::exit(1);
        }
    };

    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let mut stderr = io::stderr().lock();
        writeln!(stderr, "{diagnostics}").ok();
        writeln!(
            stderr,
            "✗ invalid  {}  ({error_count} error(s))",
            args.input.display()
        )
        .ok();
        std::process::exit(1);
    }

    println!("✓ valid  {}", args.input.display());
    Ok(())
}
