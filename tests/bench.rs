use std::path::PathBuf;

use llm_format::bench::tokenizer::{DEFAULT_ENCODING_NAME, count_tokens};
use llm_format::bench::{BenchReport, measure_document};
use llm_format::cli::bench::{BenchArgs, execute};
use llm_format::transpile::{self, Target};
use llm_format::{parse_str, validate_document};

fn parse_valid_document(source: &str) -> llm_format::Document {
    let document = parse_str(source).expect("fixture should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors, got: {diagnostics}"
    );
    document
}

fn assert_report_matches_representations(source: &str, report: &BenchReport) {
    let document = parse_valid_document(source);
    let representations = [
        ("source", source.to_owned()),
        ("plain", transpile::transpile(&document, Target::Plain)),
        ("json-ir", transpile::transpile(&document, Target::JsonIr)),
        ("shadow", transpile::transpile(&document, Target::Shadow)),
    ];

    assert_eq!(report.tokenizer, DEFAULT_ENCODING_NAME);
    assert_eq!(report.rows.len(), representations.len());

    let baseline_bytes = source.len() as i128;
    let baseline_tokens = count_tokens(source).expect("token count should succeed") as i128;

    for (row, (expected_name, expected_text)) in report.rows.iter().zip(representations.iter()) {
        assert_eq!(row.name, *expected_name);
        assert_eq!(row.bytes, expected_text.len());
        assert_eq!(
            row.tokens,
            count_tokens(expected_text).expect("token count should succeed")
        );
        assert_eq!(row.delta_bytes, row.bytes as i128 - baseline_bytes);
        assert_eq!(row.delta_tokens, row.tokens as i128 - baseline_tokens);
    }
}

#[test]
fn bench_report_measures_minimal_fixture_representations() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let report = measure_document(source, &document).expect("bench report should build");

    assert_report_matches_representations(source, &report);
}

#[test]
fn bench_report_measures_quoted_fixture_representations() {
    let source = include_str!("../examples/quoted.llm");
    let document = parse_valid_document(source);
    let report = measure_document(source, &document).expect("bench report should build");

    assert_report_matches_representations(source, &report);
}

#[test]
fn bench_cli_output_is_deterministic() {
    let first = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
    })
    .expect("bench execution should succeed");
    let second = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
    })
    .expect("bench execution should succeed");

    assert_eq!(first, second);
    assert!(first.contains("tokenizer: cl100k_base"));
    assert!(first.contains("source  | bytes="));
    assert!(first.contains("plain   | bytes="));
    assert!(first.contains("json-ir | bytes="));
    assert!(first.contains("shadow  | bytes="));
}

#[test]
fn bench_cli_reports_quoted_fixture_structure() {
    let report = execute(BenchArgs {
        input: PathBuf::from("examples/quoted.llm"),
    })
    .expect("bench execution should succeed");

    assert!(report.contains("tokenizer: cl100k_base"));
    assert!(report.contains("source  | bytes="));
    assert!(report.contains("plain   | bytes="));
    assert!(report.contains("json-ir | bytes="));
    assert!(report.contains("shadow  | bytes="));
}

#[test]
fn bench_cli_rejects_semantically_invalid_input() {
    let result = execute(BenchArgs {
        input: PathBuf::from("examples/invalid/vars-sequence.llm"),
    });

    let error = result.expect_err("invalid input should fail");
    assert!(
        error.to_string().contains("validation failed"),
        "expected validation failure, got: {error}"
    );
}
