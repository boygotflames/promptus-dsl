use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Args, ValueEnum};

use crate::parser::parse_str;
use crate::provider::Provider;
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

    #[arg(long, value_enum, default_value_t = Provider::Generic)]
    pub provider: Provider,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDestination {
    Stdout,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranspileExecution {
    pub rendered: String,
    pub destination: OutputDestination,
}

pub fn run(args: TranspileArgs) -> Result<()> {
    let execution = execute(args)?;

    if execution.destination == OutputDestination::Stdout {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(execution.rendered.as_bytes())
            .context("failed to write transpile output to stdout")?;
    }

    Ok(())
}

pub fn execute(args: TranspileArgs) -> Result<TranspileExecution> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;

    let document = parse_str(&source).map_err(|diagnostics| {
        eprintln!("{diagnostics}");
        anyhow!("parse failed")
    })?;

    let (document, compose_diags) = crate::composer::compose(document, &args.input, &[]);
    if compose_diags.has_errors() {
        eprintln!("{compose_diags}");
        return Err(anyhow!("include composition failed"));
    }

    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        eprintln!("{diagnostics}");
        return Err(anyhow!("validation failed"));
    }

    let rendered =
        transpile::transpile_with_provider(&document, args.target.into(), args.provider)?;
    let destination = match args.output {
        Some(output_path) => {
            write_output_file(&output_path, &rendered, args.force)?;
            OutputDestination::File(output_path)
        }
        None => OutputDestination::Stdout,
    };

    Ok(TranspileExecution {
        rendered,
        destination,
    })
}

fn write_output_file(path: &Path, rendered: &str, force: bool) -> Result<()> {
    if path.is_dir() {
        return Err(anyhow!("output path is a directory: {}", path.display()));
    }

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if !parent.exists() {
            return Err(anyhow!(
                "output directory does not exist: {}",
                parent.display()
            ));
        }

        if !parent.is_dir() {
            return Err(anyhow!(
                "output parent is not a directory: {}",
                parent.display()
            ));
        }
    }

    if path.exists() && !force {
        return Err(anyhow!(
            "refusing to overwrite existing output file {} (pass --force to overwrite)",
            path.display()
        ));
    }

    let mut options = OpenOptions::new();
    options.write(true);

    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }

    let mut file = options.open(path).map_err(|error| match error.kind() {
        io::ErrorKind::AlreadyExists => anyhow!(
            "refusing to overwrite existing output file {} (pass --force to overwrite)",
            path.display()
        ),
        _ => anyhow!("failed to create output file {}: {}", path.display(), error),
    })?;

    file.write_all(rendered.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))
}
