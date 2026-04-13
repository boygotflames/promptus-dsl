use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

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
        // stdin mode: no file path available, include composition skipped
        run_validation(&source, "<stdin>", None);
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
        run_validation(&source, &path.display().to_string(), Some(&path));
    }
    Ok(())
}

fn run_validation(source: &str, label: &str, source_path: Option<&Path>) {
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

    // Compose includes if a file path is available.
    // stdin mode passes None — include resolution requires a known source directory.
    let (document, compose_diags) = if let Some(path) = source_path {
        crate::composer::compose(document, path, &[])
    } else {
        (document, crate::diagnostics::DiagnosticBag::new())
    };

    // Collect compose + validate diagnostics together
    let mut all_diagnostics = compose_diags;
    let validate_diags = validate_document(&document);
    all_diagnostics.extend(validate_diags);

    if all_diagnostics.has_errors() {
        let error_count = all_diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let mut stderr = io::stderr().lock();
        writeln!(stderr, "{all_diagnostics}").ok();
        writeln!(stderr, "✗ invalid  {label}  ({error_count} error(s))").ok();
        std::process::exit(1);
    }

    println!("✓ valid  {label}");
}
