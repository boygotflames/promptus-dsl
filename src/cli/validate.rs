use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::diagnostics::Severity;
use crate::parser::parse_str;
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Input file path. Mutually exclusive with --stdin.
    #[arg(required_unless_present = "stdin")]
    pub input: Option<PathBuf>,

    /// Read document from stdin instead of a file. Mutually exclusive with INPUT.
    #[arg(long, conflicts_with = "input")]
    pub stdin: bool,
}

pub fn run(args: ValidateArgs) -> Result<()> {
    if args.stdin {
        let mut source = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut source) {
            eprintln!("error: failed to read stdin: {e}");
            std::process::exit(2);
        }
        run_validation(&source, "<stdin>");
    } else {
        let path = args
            .input
            .expect("input is required when --stdin is not set");
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: failed to read {}: {e}", path.display());
                std::process::exit(2);
            }
        };
        run_validation(&source, &path.display().to_string());
    }
    Ok(())
}

fn run_validation(source: &str, label: &str) {
    let document = match parse_str(source) {
        Ok(d) => d,
        Err(diagnostics) => {
            let error_count = diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let mut stderr = io::stderr().lock();
            writeln!(stderr, "{diagnostics}").ok();
            writeln!(stderr, "✗ invalid  {label}  ({error_count} error(s))").ok();
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
        writeln!(stderr, "✗ invalid  {label}  ({error_count} error(s))").ok();
        std::process::exit(1);
    }

    println!("✓ valid  {label}");
}
