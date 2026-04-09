use std::path::PathBuf;

use llm_format::cli::bench::{BenchArgs, execute as execute_bench};
use llm_format::provider::Provider;
use llm_format::transpile::{self, Target};
use llm_format::{DiagnosticPhase, format_document, parse_str, validate_document};

const PLAIN_MINIMAL: &str = "agent: DataExtractor\nsystem:\n  role: financial_analyst\n  output: json\nmemory:\n  - user_history";
const PLAIN_QUOTED: &str = "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  company: \"Acme Corp\"\n  region: apac";
const JSON_IR_MINIMAL: &str = "{\n  \"agent\": \"DataExtractor\",\n  \"system\": {\n    \"role\": \"financial_analyst\",\n    \"output\": \"json\"\n  },\n  \"memory\": [\n    \"user_history\"\n  ]\n}";
const SHADOW_MINIMAL: &str =
    "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]";
const SHADOW_QUOTED: &str = "@a=\"Data Extractor\"\n@s={role=\"financial analyst\"}\n@u=\"Summarize \\\"Q1\\\" results\"\n@v={company=\"Acme Corp\";region=\"apac\"}";
const FORMATTER_CANONICAL_MESSY: &str = "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  region: apac\n  company: \"Acme Corp\"";
const BENCH_GENERIC_MINIMAL: &str = "provider: generic\ntokenizer: cl100k_base\nsource  | bytes=101 | tokens=27 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=94 | tokens=26 | delta_bytes=-7 | delta_tokens=-1\njson-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19\nshadow  | bytes=82 | tokens=23 | delta_bytes=-19 | delta_tokens=-4";
const BENCH_OPENAI_MINIMAL: &str = "provider: openai\ntokenizer: cl100k_base\nsource  | bytes=101 | tokens=27 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=94 | tokens=26 | delta_bytes=-7 | delta_tokens=-1\njson-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19\nshadow  | bytes=82 | tokens=23 | delta_bytes=-19 | delta_tokens=-4";

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
fn conformance_parse_accepts_canonical_valid_fixtures() {
    for (name, fixture) in [
        ("minimal", include_str!("../examples/minimal.llm")),
        ("quoted", include_str!("../examples/quoted.llm")),
        ("extractor", include_str!("../examples/extractor.llm")),
    ] {
        parse_str(fixture).unwrap_or_else(|diagnostics| {
            panic!("expected {name} fixture to parse, got: {diagnostics}")
        });
    }
}

#[test]
fn conformance_parse_rejects_known_invalid_surface_fixtures() {
    let unknown_key = parse_str(include_str!("../examples/invalid/unknown-key.llm"))
        .expect_err("unknown key fixture should fail");
    assert_eq!(
        unknown_key.to_string(),
        "syntax error at 2:1: unknown top-level key `persona`"
    );

    let missing_colon = parse_str(include_str!("../examples/invalid/missing-colon.llm"))
        .expect_err("missing colon fixture should fail");
    assert_eq!(
        missing_colon.to_string(),
        "syntax error at 1:6: expected `:` after mapping key"
    );

    let duplicate_top_level =
        parse_str(include_str!("../examples/invalid/duplicate-top-level.llm"))
            .expect_err("duplicate top-level fixture should fail");
    assert_eq!(
        duplicate_top_level.to_string(),
        "syntax error at 2:1: duplicate top-level key `agent`"
    );
}

#[test]
fn conformance_validation_rejects_known_semantic_failures() {
    let vars_fixture = parse_str(include_str!("../examples/invalid/vars-sequence.llm"))
        .expect("vars-sequence fixture should parse before validation");
    let vars_diagnostics = validate_document(&vars_fixture);
    let vars_error = vars_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("`vars`"))
        .expect("expected vars validation diagnostic");
    assert_eq!(vars_error.phase, DiagnosticPhase::Semantic);
    assert_eq!(
        vars_error.to_string(),
        "semantic error at 2:1: `vars` must be a mapping of scalar values"
    );

    let duplicate_mapping_source = r#"
agent: DataExtractor
system:
  role: first
  role: second
"#;
    let duplicate_mapping_document =
        parse_str(duplicate_mapping_source).expect("duplicate mapping fixture should parse");
    let duplicate_mapping_diagnostics = validate_document(&duplicate_mapping_document);
    let duplicate_mapping_error = duplicate_mapping_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("duplicate key `role`"))
        .expect("expected duplicate-key diagnostic");
    assert_eq!(duplicate_mapping_error.phase, DiagnosticPhase::Semantic);
    assert_eq!(
        duplicate_mapping_error.to_string(),
        "semantic error at 5:3: duplicate key `role` in `system`"
    );
}

#[test]
fn conformance_formatter_is_canonical_and_idempotent() {
    let minimal_document = parse_valid_document(include_str!("../examples/minimal.llm"));
    assert_eq!(format_document(&minimal_document), PLAIN_MINIMAL);

    let quoted_document = parse_valid_document(include_str!("../examples/quoted.llm"));
    assert_eq!(format_document(&quoted_document), PLAIN_QUOTED);

    let first = format_document(&parse_valid_document(include_str!(
        "../examples/noncanonical/messy.llm"
    )));
    assert_eq!(first, FORMATTER_CANONICAL_MESSY);

    let second = format_document(&parse_valid_document(&first));
    assert_eq!(first, second);
}

#[test]
fn conformance_plain_and_json_ir_outputs_are_deterministic() {
    let document = parse_valid_document(include_str!("../examples/minimal.llm"));

    let first_plain = transpile::transpile(&document, Target::Plain);
    let second_plain = transpile::transpile(&document, Target::Plain);
    assert_eq!(first_plain, second_plain);
    assert_eq!(first_plain, PLAIN_MINIMAL);

    let first_json_ir = transpile::transpile(&document, Target::JsonIr);
    let second_json_ir = transpile::transpile(&document, Target::JsonIr);
    assert_eq!(first_json_ir, second_json_ir);
    assert_eq!(first_json_ir, JSON_IR_MINIMAL);
}

#[test]
fn conformance_shadow_output_is_deterministic_for_supported_providers() {
    let minimal_document = parse_valid_document(include_str!("../examples/minimal.llm"));
    let quoted_document = parse_valid_document(include_str!("../examples/quoted.llm"));

    let default_shadow = transpile::transpile(&minimal_document, Target::Shadow);
    let generic_shadow =
        transpile::transpile_with_provider(&minimal_document, Target::Shadow, Provider::Generic)
            .expect("generic shadow transpilation should succeed");
    let openai_shadow =
        transpile::transpile_with_provider(&minimal_document, Target::Shadow, Provider::Openai)
            .expect("openai shadow transpilation should succeed");

    assert_eq!(default_shadow, SHADOW_MINIMAL);
    assert_eq!(generic_shadow, SHADOW_MINIMAL);
    assert_eq!(openai_shadow, SHADOW_MINIMAL);
    assert_eq!(default_shadow, generic_shadow);

    let quoted_shadow = transpile::transpile(&quoted_document, Target::Shadow);
    assert_eq!(quoted_shadow, SHADOW_QUOTED);
}

#[test]
fn conformance_shadow_rejects_unsupported_provider_profiles_explicitly() {
    let document = parse_valid_document(include_str!("../examples/minimal.llm"));
    let error = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Anthropic)
        .expect_err("unsupported shadow provider should fail");

    assert_eq!(
        error.to_string(),
        "provider anthropic does not have a supported shadow profile yet"
    );
}

#[test]
fn conformance_bench_reports_are_deterministic_for_supported_providers() {
    let first_generic = execute_bench(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Generic,
        baseline: None,
    })
    .expect("generic bench execution should succeed");
    let second_generic = execute_bench(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Generic,
        baseline: None,
    })
    .expect("generic bench execution should succeed");
    assert_eq!(first_generic, second_generic);
    assert_eq!(first_generic, BENCH_GENERIC_MINIMAL);

    let first_openai = execute_bench(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Openai,
        baseline: None,
    })
    .expect("openai bench execution should succeed");
    let second_openai = execute_bench(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Openai,
        baseline: None,
    })
    .expect("openai bench execution should succeed");
    assert_eq!(first_openai, second_openai);
    assert_eq!(first_openai, BENCH_OPENAI_MINIMAL);
}

#[test]
fn conformance_bench_rejects_unsupported_provider_profiles_explicitly() {
    let error = execute_bench(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect_err("unsupported bench provider should fail");

    assert_eq!(
        error.to_string(),
        "provider anthropic does not have a supported tokenizer profile yet"
    );
}
