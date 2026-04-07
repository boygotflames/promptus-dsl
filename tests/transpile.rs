use std::path::PathBuf;

use llm_format::cli::transpile::{TargetArg, TranspileArgs};
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

#[test]
fn plain_transpile_matches_minimal_fixture() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile(&document, Target::Plain);
    assert_eq!(
        rendered,
        "agent: DataExtractor\nsystem:\n  role: financial_analyst\n  output: json\nmemory:\n  - user_history"
    );
}

#[test]
fn json_ir_transpile_matches_minimal_fixture() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile(&document, Target::JsonIr);

    assert_eq!(
        rendered,
        "{\n  \"agent\": \"DataExtractor\",\n  \"system\": {\n    \"role\": \"financial_analyst\",\n    \"output\": \"json\"\n  },\n  \"memory\": [\n    \"user_history\"\n  ]\n}"
    );
}

#[test]
fn plain_transpile_quotes_non_bare_scalars_in_quoted_fixture() {
    let source = include_str!("../examples/quoted.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile(&document, Target::Plain);

    assert_eq!(
        rendered,
        "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  company: \"Acme Corp\"\n  region: apac"
    );
}

#[test]
fn json_ir_transpile_matches_quoted_fixture() {
    let source = include_str!("../examples/quoted.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile(&document, Target::JsonIr);

    assert_eq!(
        rendered,
        "{\n  \"agent\": \"Data Extractor\",\n  \"system\": {\n    \"role\": \"financial analyst\"\n  },\n  \"user\": \"Summarize \\\"Q1\\\" results\",\n  \"vars\": {\n    \"company\": \"Acme Corp\",\n    \"region\": \"apac\"\n  }\n}"
    );
}

#[test]
fn transpile_outputs_are_deterministic_across_repeated_calls() {
    let source = include_str!("../examples/quoted.llm");
    let document = parse_valid_document(source);

    let first_plain = transpile::transpile(&document, Target::Plain);
    let second_plain = transpile::transpile(&document, Target::Plain);
    let first_json = transpile::transpile(&document, Target::JsonIr);
    let second_json = transpile::transpile(&document, Target::JsonIr);
    let first_shadow = transpile::transpile(&document, Target::Shadow);
    let second_shadow = transpile::transpile(&document, Target::Shadow);

    assert_eq!(first_plain, second_plain);
    assert_eq!(first_json, second_json);
    assert_eq!(first_shadow, second_shadow);
}

#[test]
fn transpile_cli_rejects_semantically_invalid_input() {
    let result = llm_format::cli::transpile::run(TranspileArgs {
        input: PathBuf::from("examples/invalid/vars-sequence.llm"),
        target: TargetArg::Shadow,
    });

    assert!(result.is_err(), "expected transpile CLI path to fail");
    let message = result.expect_err("result should be an error").to_string();
    assert!(
        message.contains("validation failed"),
        "expected validation failure, got: {message}"
    );
}

#[test]
fn shadow_transpile_matches_minimal_fixture() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile(&document, Target::Shadow);

    assert_eq!(
        rendered,
        "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]"
    );
}

#[test]
fn shadow_transpile_matches_quoted_fixture() {
    let source = include_str!("../examples/quoted.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile(&document, Target::Shadow);

    assert_eq!(
        rendered,
        "@a=\"Data Extractor\"\n@s={role=\"financial analyst\"}\n@u=\"Summarize \\\"Q1\\\" results\"\n@v={company=\"Acme Corp\";region=\"apac\"}"
    );
}
