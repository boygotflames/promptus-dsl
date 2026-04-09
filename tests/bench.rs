use std::path::PathBuf;

use llm_format::bench::tokenizer::{DEFAULT_ENCODING_NAME, count_tokens};
use llm_format::bench::{
    BASELINE_ROW_NAME, BenchReport, measure_document, measure_document_with_baseline,
    measure_document_with_provider, measure_document_with_provider_and_baseline,
};
use llm_format::cli::bench::{BenchArgs, execute};
use llm_format::provider::Provider;
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

    assert_eq!(report.provider, Provider::Generic.as_str());
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
        assert_eq!(row.delta_bytes_vs_baseline, None);
        assert_eq!(row.delta_tokens_vs_baseline, None);
    }
}

fn assert_report_matches_representations_with_baseline(
    source: &str,
    baseline: &str,
    report: &BenchReport,
) {
    let document = parse_valid_document(source);
    let representations = [
        ("source", source.to_owned()),
        (BASELINE_ROW_NAME, baseline.to_owned()),
        ("plain", transpile::transpile(&document, Target::Plain)),
        ("json-ir", transpile::transpile(&document, Target::JsonIr)),
        ("shadow", transpile::transpile(&document, Target::Shadow)),
    ];

    assert_eq!(report.provider, Provider::Generic.as_str());
    assert_eq!(report.tokenizer, DEFAULT_ENCODING_NAME);
    assert_eq!(report.rows.len(), representations.len());

    let source_bytes = source.len() as i128;
    let source_tokens = count_tokens(source).expect("token count should succeed") as i128;
    let baseline_bytes = baseline.len() as i128;
    let baseline_tokens = count_tokens(baseline).expect("token count should succeed") as i128;

    for (row, (expected_name, expected_text)) in report.rows.iter().zip(representations.iter()) {
        assert_eq!(row.name, *expected_name);
        assert_eq!(row.bytes, expected_text.len());
        assert_eq!(
            row.tokens,
            count_tokens(expected_text).expect("token count should succeed")
        );
        assert_eq!(row.delta_bytes, row.bytes as i128 - source_bytes);
        assert_eq!(row.delta_tokens, row.tokens as i128 - source_tokens);
        assert_eq!(
            row.delta_bytes_vs_baseline,
            Some(row.bytes as i128 - baseline_bytes)
        );
        assert_eq!(
            row.delta_tokens_vs_baseline,
            Some(row.tokens as i128 - baseline_tokens)
        );
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
        provider: Provider::Generic,
        baseline: None,
    })
    .expect("bench execution should succeed");
    let second = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Generic,
        baseline: None,
    })
    .expect("bench execution should succeed");

    assert_eq!(first, second);
    assert_eq!(
        first,
        "provider: generic\ntokenizer: cl100k_base\nsource  | bytes=101 | tokens=27 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=94 | tokens=26 | delta_bytes=-7 | delta_tokens=-1\njson-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19\nshadow  | bytes=82 | tokens=23 | delta_bytes=-19 | delta_tokens=-4"
    );
}

#[test]
fn bench_cli_reports_quoted_fixture_structure() {
    let report = execute(BenchArgs {
        input: PathBuf::from("examples/quoted.llm"),
        provider: Provider::Generic,
        baseline: None,
    })
    .expect("bench execution should succeed");

    assert!(report.contains("provider: generic"));
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
        provider: Provider::Generic,
        baseline: None,
    });

    let error = result.expect_err("invalid input should fail");
    assert!(
        error.to_string().contains("validation failed"),
        "expected validation failure, got: {error}"
    );
}

#[test]
fn bench_report_measures_minimal_fixture_with_baseline() {
    let source = include_str!("../examples/minimal.llm");
    let baseline = include_str!("../examples/baselines/minimal.md");
    let document = parse_valid_document(source);
    let report = measure_document_with_baseline(source, &document, Some(baseline))
        .expect("bench report with baseline should build");

    assert_report_matches_representations_with_baseline(source, baseline, &report);
}

#[test]
fn bench_report_measures_quoted_fixture_with_baseline() {
    let source = include_str!("../examples/quoted.llm");
    let baseline = include_str!("../examples/baselines/quoted.md");
    let document = parse_valid_document(source);
    let report = measure_document_with_baseline(source, &document, Some(baseline))
        .expect("bench report with baseline should build");

    assert_report_matches_representations_with_baseline(source, baseline, &report);
}

#[test]
fn bench_cli_output_is_deterministic_with_baseline() {
    let first = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/minimal.md")),
    })
    .expect("bench execution with baseline should succeed");
    let second = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/minimal.md")),
    })
    .expect("bench execution with baseline should succeed");

    assert_eq!(first, second);
    assert!(first.contains("provider: generic"));
    assert!(first.contains("baseline | bytes="));
    assert!(first.contains("delta_bytes_vs_baseline="));
    assert!(first.contains("delta_tokens_vs_baseline="));
}

#[test]
fn bench_cli_reports_missing_baseline_files() {
    let result = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/does-not-exist.md")),
    });

    let error = result.expect_err("missing baseline file should fail");
    assert!(
        error.to_string().contains("failed to read baseline"),
        "expected baseline read failure, got: {error}"
    );
}

#[test]
fn bench_report_matches_explicit_openai_provider() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let report = measure_document_with_provider(source, &document, Provider::Openai)
        .expect("openai bench report should build");

    assert_eq!(report.provider, "openai");
    assert_eq!(report.tokenizer, DEFAULT_ENCODING_NAME);
    assert_eq!(report.rows[0].name, "source");
}

#[test]
fn bench_report_matches_explicit_openai_provider_with_baseline() {
    let source = include_str!("../examples/minimal.llm");
    let baseline = include_str!("../examples/baselines/minimal.md");
    let document = parse_valid_document(source);
    let report = measure_document_with_provider_and_baseline(
        source,
        &document,
        Provider::Openai,
        Some(baseline),
    )
    .expect("openai bench report with baseline should build");

    assert_eq!(report.provider, "openai");
    assert_eq!(report.tokenizer, DEFAULT_ENCODING_NAME);
    assert_eq!(report.rows[1].name, BASELINE_ROW_NAME);
}

#[test]
fn bench_cli_reports_openai_provider_selection() {
    let report = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Openai,
        baseline: None,
    })
    .expect("bench execution should succeed for openai provider");

    assert!(report.contains("provider: openai"));
    assert!(report.contains("tokenizer: cl100k_base"));
}

#[test]
fn bench_cli_output_is_deterministic_for_openai_provider() {
    let first = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Openai,
        baseline: None,
    })
    .expect("bench execution should succeed for openai provider");
    let second = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Openai,
        baseline: None,
    })
    .expect("bench execution should succeed for openai provider");

    assert_eq!(first, second);
}

#[test]
fn bench_cli_rejects_unsupported_provider_selection() {
    let result = execute(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    });

    let error = result.expect_err("unsupported provider should fail");
    assert!(
        error
            .to_string()
            .contains("provider anthropic does not have a supported tokenizer profile yet"),
        "expected unsupported provider/tokenizer message, got: {error}"
    );
}
