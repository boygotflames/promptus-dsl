use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use llm_format::cli::fmt::{FmtArgs, OutputDestination, execute};
use llm_format::{format_document, parse_str, validate_document};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn parse_valid_document(source: &str) -> llm_format::Document {
    let document = parse_str(source).expect("fixture should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors, got: {diagnostics}"
    );
    document
}

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "llm_format_fmt_{label}_{}_{}",
            std::process::id(),
            TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));

        if path.exists() {
            fs::remove_dir_all(&path).expect("stale temp directory should be removable");
        }

        fs::create_dir_all(&path).expect("temp directory should be creatable");
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn formatter_matches_minimal_fixture() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);

    assert_eq!(
        format_document(&document),
        "agent: DataExtractor\nsystem:\n  role: financial_analyst\n  output: json\nmemory:\n  - user_history"
    );
}

#[test]
fn formatter_matches_quoted_fixture() {
    let source = include_str!("../examples/quoted.llm");
    let document = parse_valid_document(source);

    assert_eq!(
        format_document(&document),
        "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  company: \"Acme Corp\"\n  region: apac"
    );
}

#[test]
fn formatter_normalizes_noncanonical_fixture() {
    let source = include_str!("../examples/noncanonical/messy.llm");
    let document = parse_valid_document(source);

    assert_eq!(
        format_document(&document),
        "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  region: apac\n  company: \"Acme Corp\""
    );
}

#[test]
fn formatter_is_idempotent() {
    let source = include_str!("../examples/noncanonical/messy.llm");
    let first_document = parse_valid_document(source);
    let first = format_document(&first_document);
    let second_document = parse_valid_document(&first);
    let second = format_document(&second_document);

    assert_eq!(first, second);
}

#[test]
fn fmt_execute_returns_stdout_payload_by_default() {
    let execution = execute(FmtArgs {
        input: PathBuf::from("examples/minimal.llm"),
        write: false,
    })
    .expect("fmt execution should succeed");

    assert_eq!(execution.destination, OutputDestination::Stdout);
    assert_eq!(
        execution.rendered,
        "agent: DataExtractor\nsystem:\n  role: financial_analyst\n  output: json\nmemory:\n  - user_history"
    );
}

#[test]
fn fmt_execute_writes_back_to_input_when_requested() {
    let temp_dir = TestTempDir::new("write_back");
    let input_path = temp_dir.path().join("messy.llm");
    fs::write(
        &input_path,
        include_str!("../examples/noncanonical/messy.llm"),
    )
    .expect("temp fixture should be writable");

    let execution = execute(FmtArgs {
        input: input_path.clone(),
        write: true,
    })
    .expect("write-back execution should succeed");

    assert_eq!(
        execution.destination,
        OutputDestination::Write(input_path.clone())
    );
    assert_eq!(
        fs::read_to_string(&input_path).expect("formatted file should be readable"),
        "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  region: apac\n  company: \"Acme Corp\""
    );
}

#[test]
fn fmt_execute_rejects_invalid_input() {
    let result = execute(FmtArgs {
        input: PathBuf::from("examples/invalid/vars-sequence.llm"),
        write: false,
    });

    let error = result.expect_err("invalid input should fail");
    assert!(
        error.to_string().contains("validation failed"),
        "expected validation failure, got: {error}"
    );
}
