use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::ast::{Node, TopLevelKey};
use crate::diagnostics::Severity;
use crate::parser::parse_str;

#[derive(Debug, Args)]
pub struct ParseArgs {
    pub input: PathBuf,

    /// Print a compact key/node summary instead of the full AST
    #[arg(long)]
    pub summary: bool,
}

fn count_nodes(node: &Node) -> usize {
    match node {
        Node::Scalar { .. } => 1,
        Node::Sequence { values, .. } => 1 + values.iter().map(count_nodes).sum::<usize>(),
        Node::Mapping { entries, .. } => {
            1 + entries.iter().map(|e| count_nodes(&e.value)).sum::<usize>()
        }
    }
}

pub fn run(args: ParseArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;

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
                "✗ parse failed  {}  ({error_count} error(s))",
                args.input.display()
            )
            .ok();
            std::process::exit(1);
        }
    };

    if args.summary {
        let keys: Vec<&str> = TopLevelKey::ordered()
            .into_iter()
            .filter(|&key| document.get(key).is_some())
            .map(|key| key.as_str())
            .collect();

        let total_nodes: usize = TopLevelKey::ordered()
            .into_iter()
            .filter_map(|key| document.get(key))
            .map(count_nodes)
            .sum();

        println!("✓ parsed  {}", args.input.display());
        println!("keys: {}", keys.join(" "));
        println!("nodes: {total_nodes}");
    } else {
        println!("{document:#?}");
    }

    Ok(())
}
