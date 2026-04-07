use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod bench;
pub mod fmt;
pub mod parse;
pub mod transpile;
pub mod validate;

#[derive(Debug, Parser)]
#[command(name = "llm-format", about = "Compiler foundation for the .llm format")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Parse(parse::ParseArgs),
    Validate(validate::ValidateArgs),
    Transpile(transpile::TranspileArgs),
    Fmt(fmt::FmtArgs),
    Bench(bench::BenchArgs),
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Parse(args) => parse::run(args),
        Command::Validate(args) => validate::run(args),
        Command::Transpile(args) => transpile::run(args),
        Command::Fmt(args) => fmt::run(args),
        Command::Bench(args) => bench::run(args),
    }
}
