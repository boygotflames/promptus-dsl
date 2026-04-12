use std::path::PathBuf;

use llm_format::cli::bench::{BenchArgs, execute as execute_bench};
use llm_format::provider::Provider;
use llm_format::transpile::{self, Target};
use llm_format::{DiagnosticPhase, TopLevelKey, format_document, parse_str, validate_document};

const SHADOW_EXTRACTOR: &str = "@a=\"Extractor\"\n@s={role=\"financial_analyst\";objective=\"extract structured facts\"}\n@u={prompt=\"summarize the quarterly filing\"}\n@t=[\"web_search\",\"calculator\"]\n@c=[\"cite_sources\",\"stay_provider_agnostic\"]";
const SHADOW_JSON_OUTPUT: &str = "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@o={schema=\"invoice_summary\"}\n@v={region=\"apac\";currency=\"usd\"}";

const PLAIN_MINIMAL: &str = "agent: DataExtractor\nsystem:\n  role: financial_analyst\n  output: json\nmemory:\n  - user_history";
const PLAIN_QUOTED: &str = "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  company: \"Acme Corp\"\n  region: apac";
const JSON_IR_MINIMAL: &str = "{\n  \"agent\": \"DataExtractor\",\n  \"system\": {\n    \"role\": \"financial_analyst\",\n    \"output\": \"json\"\n  },\n  \"memory\": [\n    \"user_history\"\n  ]\n}";
const SHADOW_MINIMAL: &str =
    "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]";
const SHADOW_QUOTED: &str = "@a=\"Data Extractor\"\n@s={role=\"financial analyst\"}\n@u=\"Summarize \\\"Q1\\\" results\"\n@v={company=\"Acme Corp\";region=\"apac\"}";
const FORMATTER_CANONICAL_MESSY: &str = "agent: \"Data Extractor\"\nsystem:\n  role: \"financial analyst\"\nuser: \"Summarize \\\"Q1\\\" results\"\nvars:\n  region: apac\n  company: \"Acme Corp\"";
const BENCH_GENERIC_MINIMAL: &str = "provider: generic\ntokenizer: cl100k_base\nsource  | bytes=101 | tokens=27 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=94 | tokens=26 | delta_bytes=-7 | delta_tokens=-1\njson-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19\nshadow  | bytes=82 | tokens=23 | delta_bytes=-19 | delta_tokens=-4";
const BENCH_OPENAI_MINIMAL: &str = "provider: openai\ntokenizer: cl100k_base\nsource  | bytes=101 | tokens=27 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=94 | tokens=26 | delta_bytes=-7 | delta_tokens=-1\njson-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19\nshadow  | bytes=82 | tokens=23 | delta_bytes=-19 | delta_tokens=-4";
const BENCH_GENERIC_EXTRACTOR_BASELINE: &str = "provider: generic\ntokenizer: cl100k_base\nsource   | bytes=242 | tokens=57 | delta_bytes=+0 | delta_tokens=+0 | delta_bytes_vs_baseline=+25 | delta_tokens_vs_baseline=+3\nbaseline | bytes=217 | tokens=54 | delta_bytes=-25 | delta_tokens=-3 | delta_bytes_vs_baseline=+0 | delta_tokens_vs_baseline=+0\nplain    | bytes=233 | tokens=60 | delta_bytes=-9 | delta_tokens=+3 | delta_bytes_vs_baseline=+16 | delta_tokens_vs_baseline=+6\njson-ir  | bytes=312 | tokens=89 | delta_bytes=+70 | delta_tokens=+32 | delta_bytes_vs_baseline=+95 | delta_tokens_vs_baseline=+35\nshadow   | bytes=202 | tokens=49 | delta_bytes=-40 | delta_tokens=-8 | delta_bytes_vs_baseline=-15 | delta_tokens_vs_baseline=-5";
const BENCH_GENERIC_JSON_OUTPUT_BASELINE: &str = "provider: generic\ntokenizer: cl100k_base\nsource   | bytes=150 | tokens=42 | delta_bytes=+0 | delta_tokens=+0 | delta_bytes_vs_baseline=+7 | delta_tokens_vs_baseline=+1\nbaseline | bytes=143 | tokens=41 | delta_bytes=-7 | delta_tokens=-1 | delta_bytes_vs_baseline=+0 | delta_tokens_vs_baseline=+0\nplain    | bytes=140 | tokens=41 | delta_bytes=-10 | delta_tokens=-1 | delta_bytes_vs_baseline=-3 | delta_tokens_vs_baseline=+0\njson-ir  | bytes=215 | tokens=72 | delta_bytes=+65 | delta_tokens=+30 | delta_bytes_vs_baseline=+72 | delta_tokens_vs_baseline=+31\nshadow   | bytes=126 | tokens=39 | delta_bytes=-24 | delta_tokens=-3 | delta_bytes_vs_baseline=-17 | delta_tokens_vs_baseline=-2";

const SHADOW_V1_MINIMAL: &str = "<agent>DataExtractor</agent>\n<system>\nrole: financial_analyst\noutput: json\n</system>\n<memory>\n<item>user_history</item>\n</memory>";
const SHADOW_V1_EXTRACTOR: &str = "<agent>Extractor</agent>\n<system>\nrole: financial_analyst\nobjective: extract structured facts\n</system>\n<user>\nprompt: summarize the quarterly filing\n</user>\n<tools>\n<tool>web_search</tool>\n<tool>calculator</tool>\n</tools>\n<constraints>\n<rule>cite_sources</rule>\n<rule>stay_provider_agnostic</rule>\n</constraints>";
const BENCH_ANTHROPIC_MINIMAL: &str = "provider: anthropic\ntokenizer: o200k_base\nsource  | bytes=101 | tokens=27 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=94 | tokens=26 | delta_bytes=-7 | delta_tokens=-1\njson-ir | bytes=141 | tokens=46 | delta_bytes=+40 | delta_tokens=+19\nshadow  | bytes=129 | tokens=39 | delta_bytes=+28 | delta_tokens=+12";
const BENCH_ANTHROPIC_EXTRACTOR: &str = "provider: anthropic\ntokenizer: o200k_base\nsource  | bytes=242 | tokens=58 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=233 | tokens=61 | delta_bytes=-9 | delta_tokens=+3\njson-ir | bytes=312 | tokens=90 | delta_bytes=+70 | delta_tokens=+32\nshadow  | bytes=313 | tokens=85 | delta_bytes=+71 | delta_tokens=+27";
const BENCH_ANTHROPIC_JSON_OUTPUT: &str = "provider: anthropic\ntokenizer: o200k_base\nsource  | bytes=150 | tokens=41 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=140 | tokens=40 | delta_bytes=-10 | delta_tokens=-1\njson-ir | bytes=215 | tokens=71 | delta_bytes=+65 | delta_tokens=+30\nshadow  | bytes=169 | tokens=51 | delta_bytes=+19 | delta_tokens=+10";
const BENCH_ANTHROPIC_QUOTED: &str = "provider: anthropic\ntokenizer: o200k_base\nsource  | bytes=142 | tokens=44 | delta_bytes=+0 | delta_tokens=+0\nplain   | bytes=136 | tokens=43 | delta_bytes=-6 | delta_tokens=-1\njson-ir | bytes=186 | tokens=64 | delta_bytes=+44 | delta_tokens=+20\nshadow  | bytes=155 | tokens=51 | delta_bytes=+13 | delta_tokens=+7";

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
        "syntax error at 2:1: [E013] unknown top-level key `persona`"
    );

    let missing_colon = parse_str(include_str!("../examples/invalid/missing-colon.llm"))
        .expect_err("missing colon fixture should fail");
    assert_eq!(
        missing_colon.to_string(),
        "syntax error at 1:6: [E006] expected `:` after mapping key"
    );

    let duplicate_top_level =
        parse_str(include_str!("../examples/invalid/duplicate-top-level.llm"))
            .expect_err("duplicate top-level fixture should fail");
    assert_eq!(
        duplicate_top_level.to_string(),
        "syntax error at 2:1: [E014] duplicate top-level key `agent`"
    );
}

#[test]
fn conformance_validation_rejects_document_without_agent_key() {
    let source = "system:\n  role: assistant";
    let document = parse_str(source).expect("missing-agent source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for document missing the agent key"
    );
    assert!(
        diagnostics.iter().any(|d| d.message.contains("agent")),
        "expected diagnostic mentioning 'agent', got: {diagnostics}"
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
        "semantic error at 2:1: [E109] `vars` must be a mapping of scalar values"
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
        "semantic error at 5:3: [E102] duplicate key `role` in `system`"
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
fn conformance_anthropic_shadow_produces_v1_xml_encoding() {
    let document = parse_valid_document(include_str!("../examples/minimal.llm"));
    let output = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Anthropic)
        .expect("anthropic shadow should succeed");
    assert_eq!(output, SHADOW_V1_MINIMAL);
}

#[test]
fn conformance_anthropic_shadow_differs_from_generic_v0() {
    let document = parse_valid_document(include_str!("../examples/minimal.llm"));
    let v0 = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Generic)
        .expect("generic shadow should succeed");
    let v1 = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Anthropic)
        .expect("anthropic shadow should succeed");
    assert_ne!(v0, v1, "V0 and V1Anthropic shadow output must differ");
}

#[test]
fn conformance_anthropic_shadow_extractor_matches_v1_contract() {
    let document = parse_valid_document(include_str!("../examples/extractor.llm"));
    let output = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Anthropic)
        .expect("anthropic extractor shadow should succeed");
    assert_eq!(output, SHADOW_V1_EXTRACTOR);
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
fn conformance_bench_anthropic_provider_is_supported() {
    let result = execute_bench(BenchArgs {
        input: PathBuf::from("examples/minimal.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic bench should succeed");
    assert_eq!(result, BENCH_ANTHROPIC_MINIMAL);
}

#[test]
fn conformance_bench_anthropic_extractor_is_deterministic() {
    let first = execute_bench(BenchArgs {
        input: PathBuf::from("examples/extractor.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic extractor bench should succeed");
    let second = execute_bench(BenchArgs {
        input: PathBuf::from("examples/extractor.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic extractor bench should succeed");

    assert_eq!(first, second);
    assert_eq!(first, BENCH_ANTHROPIC_EXTRACTOR);
}

#[test]
fn conformance_bench_anthropic_json_output_is_deterministic() {
    let first = execute_bench(BenchArgs {
        input: PathBuf::from("examples/json-output.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic json-output bench should succeed");
    let second = execute_bench(BenchArgs {
        input: PathBuf::from("examples/json-output.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic json-output bench should succeed");

    assert_eq!(first, second);
    assert_eq!(first, BENCH_ANTHROPIC_JSON_OUTPUT);
}

#[test]
fn conformance_bench_anthropic_quoted_is_deterministic() {
    let first = execute_bench(BenchArgs {
        input: PathBuf::from("examples/quoted.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic quoted bench should succeed");
    let second = execute_bench(BenchArgs {
        input: PathBuf::from("examples/quoted.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic quoted bench should succeed");

    assert_eq!(first, second);
    assert_eq!(first, BENCH_ANTHROPIC_QUOTED);
}

#[test]
fn conformance_bench_anthropic_token_counts_differ_from_generic() {
    let generic = execute_bench(BenchArgs {
        input: PathBuf::from("examples/extractor.llm"),
        provider: Provider::Generic,
        baseline: None,
    })
    .expect("generic extractor bench should succeed");
    let anthropic = execute_bench(BenchArgs {
        input: PathBuf::from("examples/extractor.llm"),
        provider: Provider::Anthropic,
        baseline: None,
    })
    .expect("anthropic extractor bench should succeed");

    assert_ne!(
        generic, anthropic,
        "generic and anthropic bench output must differ (different tokenizer + shadow profiles)"
    );
}

#[test]
fn conformance_bench_extractor_baseline_is_deterministic() {
    let first = execute_bench(BenchArgs {
        input: PathBuf::from("examples/extractor.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/extractor.md")),
    })
    .expect("extractor bench with baseline should succeed");
    let second = execute_bench(BenchArgs {
        input: PathBuf::from("examples/extractor.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/extractor.md")),
    })
    .expect("extractor bench with baseline should succeed");

    assert_eq!(first, second);
    assert_eq!(first, BENCH_GENERIC_EXTRACTOR_BASELINE);
}

#[test]
fn conformance_bench_json_output_baseline_is_deterministic() {
    let first = execute_bench(BenchArgs {
        input: PathBuf::from("examples/json-output.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/json-output.md")),
    })
    .expect("json-output bench with baseline should succeed");
    let second = execute_bench(BenchArgs {
        input: PathBuf::from("examples/json-output.llm"),
        provider: Provider::Generic,
        baseline: Some(PathBuf::from("examples/baselines/json-output.md")),
    })
    .expect("json-output bench with baseline should succeed");

    assert_eq!(first, second);
    assert_eq!(first, BENCH_GENERIC_JSON_OUTPUT_BASELINE);
}

#[test]
fn conformance_bench_savings_are_positive_for_all_fixtures() {
    // Shadow must be cheaper than the honest Markdown baseline for every fixture.
    // delta_tokens_vs_baseline < 0 means shadow used fewer tokens than baseline.
    let fixtures: &[(&str, &str)] = &[
        ("examples/minimal.llm", "examples/baselines/minimal.md"),
        ("examples/extractor.llm", "examples/baselines/extractor.md"),
        (
            "examples/json-output.llm",
            "examples/baselines/json-output.md",
        ),
        ("examples/quoted.llm", "examples/baselines/quoted.md"),
    ];

    for (source_path, baseline_path) in fixtures {
        let report = execute_bench(BenchArgs {
            input: PathBuf::from(source_path),
            provider: Provider::Generic,
            baseline: Some(PathBuf::from(baseline_path)),
        })
        .unwrap_or_else(|e| panic!("bench failed for {source_path}: {e}"));

        assert!(
            report.contains("shadow") && report.contains("delta_tokens_vs_baseline=-"),
            "expected shadow to save tokens vs baseline for {source_path}, got:\n{report}"
        );
    }
}

// --- Stable shadow contract conformance ---

#[test]
fn conformance_shadow_extractor_fixture_matches_stable_contract() {
    let document = parse_valid_document(include_str!("../examples/extractor.llm"));
    let shadow = transpile::transpile(&document, Target::Shadow);
    assert_eq!(shadow, SHADOW_EXTRACTOR);
}

#[test]
fn conformance_shadow_json_output_fixture_covers_output_and_vars_markers() {
    let document = parse_valid_document(include_str!("../examples/json-output.llm"));
    let shadow = transpile::transpile(&document, Target::Shadow);
    assert_eq!(shadow, SHADOW_JSON_OUTPUT);
}

#[test]
fn conformance_shadow_absent_keys_are_omitted_not_emitted_empty() {
    // system is now required; use the two-key minimum to test absent-key omission
    let source = "agent: OnlyAgent\nsystem: handle requests";
    let document = parse_valid_document(source);
    let shadow = transpile::transpile(&document, Target::Shadow);
    assert_eq!(shadow, "@a=\"OnlyAgent\"\n@s=\"handle requests\"");
    assert!(
        !shadow.contains("@m"),
        "absent memory key must not appear in shadow output"
    );
    assert!(
        !shadow.contains("@t"),
        "absent tools key must not appear in shadow output"
    );
}

#[test]
fn conformance_shadow_generic_and_openai_produce_identical_output() {
    let document = parse_valid_document(include_str!("../examples/extractor.llm"));
    let generic = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Generic)
        .expect("generic shadow should succeed");
    let openai = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Openai)
        .expect("openai shadow should succeed");
    assert_eq!(generic, openai);
    assert_eq!(generic, SHADOW_EXTRACTOR);
}

// --- Diagnostic code conformance ---

#[test]
fn conformance_missing_agent_diagnostic_carries_e101_code() {
    let source = "system:\n  role: assistant";
    let document = parse_str(source).expect("missing-agent source should parse");
    let diagnostics = validate_document(&document);
    assert!(diagnostics.has_errors());
    let e101 = diagnostics
        .iter()
        .find(|d| d.code == Some("E101"))
        .expect("expected a diagnostic with code E101 for missing agent");
    assert_eq!(e101.phase, DiagnosticPhase::Semantic);
    assert!(e101.message.contains("agent"));
}

// --- vars expansion conformance ---

#[test]
fn conformance_e114_undefined_var_reference_is_rejected() {
    let document = parse_str(include_str!("../examples/invalid/undefined-var-ref.llm"))
        .expect("fixture should parse");
    let diagnostics = validate_document(&document);
    assert!(diagnostics.has_errors(), "expected E114 validation error");
    let e114 = diagnostics
        .iter()
        .find(|d| d.code == Some("E114"))
        .expect("expected E114 diagnostic");
    assert!(
        e114.message.contains("unknown_var"),
        "expected var name in E114 message, got: {}",
        e114.message
    );
}

#[test]
fn conformance_vars_expansion_in_plain_output() {
    let source = "agent: Pipeline\nsystem: \"Run query on {source}\"\nvars:\n  source: orders_raw";
    let document = parse_valid_document(source);
    let output = transpile::transpile(&document, Target::Plain);
    assert!(
        output.contains("orders_raw"),
        "expected expanded var value in plain output, got: {output}"
    );
    assert!(
        !output.contains("{source}"),
        "expected {source} to be expanded in plain output, got: {output}"
    );
}

#[test]
fn conformance_vars_expansion_in_shadow_output_v0() {
    let source = "agent: Pipeline\nsystem: \"Run query on {source}\"\nvars:\n  source: orders_raw";
    let document = parse_valid_document(source);
    let output = transpile::transpile(&document, Target::Shadow);
    assert!(
        output.contains("orders_raw"),
        "expected expanded var value in V0 shadow output, got: {output}"
    );
    assert!(
        !output.contains("{source}"),
        "expected {source} to be expanded in V0 shadow output, got: {output}"
    );
}

#[test]
fn conformance_vars_expansion_in_shadow_output_v1_anthropic() {
    let source = "agent: Pipeline\nsystem: \"Run query on {source}\"\nvars:\n  source: orders_raw";
    let document = parse_valid_document(source);
    let output = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Anthropic)
        .expect("anthropic shadow should succeed");
    assert!(
        output.contains("orders_raw"),
        "expected expanded var value in V1 anthropic shadow output, got: {output}"
    );
    assert!(
        !output.contains("{source}"),
        "expected {source} to be expanded in V1 anthropic shadow output, got: {output}"
    );
}

#[test]
fn conformance_fmt_preserves_var_references_verbatim() {
    let source = "agent: Pipeline\nsystem: \"Run query on {source}\"\nvars:\n  source: orders_raw";
    let document = parse_valid_document(source);
    let formatted = format_document(&document);
    assert!(
        formatted.contains("{source}"),
        "expected fmt to preserve {{source}} verbatim, got: {formatted}"
    );
}

#[test]
fn conformance_undefined_var_with_no_vars_block_emits_e114() {
    // No vars block at all — any {ref} is undefined
    let source = "agent: Agent\nsystem: \"Connect to {unknown}\"";
    let document = parse_str(source).expect("should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.iter().any(|d| d.code == Some("E114")),
        "expected E114 when no vars block exists"
    );
}

#[test]
fn conformance_vars_expansion_is_non_recursive() {
    // vars: a -> "{b}", b -> "final"
    // system references {a} — must expand to "{b}", not "final"
    let source = "agent: Agent\nsystem: \"value is {a}\"\nvars:\n  a: \"{b}\"\n  b: final";
    let document = parse_valid_document(source);
    let output = transpile::transpile(&document, Target::Plain);
    // {a} must expand to {b} — non-recursive, so {b} is NOT further expanded
    assert!(
        output.contains("{b}"),
        "expected non-recursive expansion to leave {{b}} unexpanded, got: {output}"
    );
    // The system line must not contain "final" (only the vars block has "final")
    let system_line = output
        .lines()
        .find(|l| l.starts_with("system:"))
        .unwrap_or("");
    assert!(
        !system_line.contains("final"),
        "expected system line to show {{b}}, not 'final', got: {system_line}"
    );
}

#[test]
fn conformance_diagnostic_codes_are_present_on_all_validator_errors() {
    // E101: missing required key: agent
    let no_agent = parse_str("system:\n  role: assistant").expect("should parse");
    let diags = validate_document(&no_agent);
    assert!(
        diags.iter().any(|d| d.code == Some("E101")),
        "expected E101 for missing agent"
    );

    // E102: duplicate mapping key
    let dup_key = parse_str("agent: X\nsystem:\n  role: a\n  role: b").expect("should parse");
    let diags = validate_document(&dup_key);
    assert!(
        diags.iter().any(|d| d.code == Some("E102")),
        "expected E102 for duplicate mapping key"
    );

    // E109: vars must be a mapping
    let vars_seq = parse_str(include_str!("../examples/invalid/vars-sequence.llm"))
        .expect("vars-sequence fixture should parse");
    let diags = validate_document(&vars_seq);
    assert!(
        diags.iter().any(|d| d.code == Some("E109")),
        "expected E109 for vars non-mapping"
    );
}

#[test]
fn conformance_validation_rejects_empty_agent_scalar() {
    let source = include_str!("../examples/invalid/empty-agent.llm");
    let document = parse_str(source).expect("empty-agent fixture should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for empty agent scalar"
    );
    let e103 = diagnostics
        .iter()
        .find(|d| d.code == Some("E103"))
        .expect("expected a diagnostic with code E103 for empty agent scalar");
    assert_eq!(e103.phase, DiagnosticPhase::Semantic);
}

#[test]
fn conformance_vars_key_grammar_is_enforced_at_parse_time() {
    // The key grammar /[A-Za-z_][A-Za-z0-9_-]*/ is enforced by the lexer (E005).
    // A vars key starting with a digit is rejected before the validator is reached.
    let diagnostics = parse_str(include_str!("../examples/invalid/vars-invalid-key.llm"))
        .expect_err("invalid key should fail to parse");
    assert_eq!(
        diagnostics.to_string(),
        "syntax error at 4:3: [E005] expected an identifier at the start of a mapping entry"
    );
}

#[test]
fn conformance_validation_rejects_document_without_system_key() {
    let source = "agent: TestAgent\n";
    let document = parse_str(source).expect("agent-only source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for document missing the system key"
    );
    let e101 = diagnostics
        .iter()
        .find(|d| d.code == Some("E101") && d.message.contains("system"))
        .expect("expected E101 diagnostic mentioning 'system'");
    assert_eq!(e101.phase, DiagnosticPhase::Semantic);
    assert_eq!(
        e101.to_string(),
        "semantic error at 1:1: [E101] missing required key: `system`"
    );
}

#[test]
fn conformance_validation_accepts_minimal_two_key_document() {
    let source = "agent: TestAgent\nsystem: Handle requests\n";
    let document = parse_str(source).expect("two-key document should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors for minimal two-key document, got: {diagnostics}"
    );
}

// --- Error code conformance: E104–E110 ---

#[test]
fn conformance_e104_non_scalar_agent_is_rejected() {
    // E104: `agent` must be a scalar value — mapping block is rejected
    let source = "agent:\n  name: TestAgent\nsystem: handle requests\n";
    let document = parse_str(source).expect("e104 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for mapping-valued agent"
    );
    let e104 = diagnostics
        .iter()
        .find(|d| d.code == Some("E104"))
        .expect("expected a diagnostic with code E104");
    assert_eq!(e104.phase, DiagnosticPhase::Semantic);
    assert!(e104.message.contains("agent"));
}

#[test]
fn conformance_e105_sequence_system_is_rejected() {
    // E105: `system` must be a scalar or mapping — sequence is rejected
    let source = "agent: TestAgent\nsystem:\n  - item\n";
    let document = parse_str(source).expect("e105 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for sequence-valued system"
    );
    let e105 = diagnostics
        .iter()
        .find(|d| d.code == Some("E105"))
        .expect("expected a diagnostic with code E105");
    assert_eq!(e105.phase, DiagnosticPhase::Semantic);
    assert!(e105.message.contains("system"));
}

#[test]
fn conformance_e106_scalar_memory_is_rejected() {
    // E106: `memory` must be a sequence — scalar value is rejected
    let source = "agent: TestAgent\nsystem: handle requests\nmemory: not-a-sequence\n";
    let document = parse_str(source).expect("e106 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for scalar-valued memory"
    );
    let e106 = diagnostics
        .iter()
        .find(|d| d.code == Some("E106"))
        .expect("expected a diagnostic with code E106");
    assert_eq!(e106.phase, DiagnosticPhase::Semantic);
    assert!(e106.message.contains("memory"));
}

// E107: sequence may only contain scalar items.
// This code is a defensive guard in validate_sequence_field; it is unreachable
// through normal parsing because the .llm parser only produces scalar nodes as
// sequence items. There is no parse path that constructs a mapping or sequence
// node inside a sequence. A conformance test cannot be written without
// constructing an AST that bypasses parse_str — which would test the validator
// in isolation, not the conformance surface. E107 is therefore documented here
// as parser-enforced and intentionally untestable at the conformance level.

#[test]
fn conformance_e108_sequence_output_is_rejected() {
    // E108: `output` must be a scalar or mapping — sequence is rejected
    let source = "agent: TestAgent\nsystem: handle requests\noutput:\n  - item\n";
    let document = parse_str(source).expect("e108 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for sequence-valued output"
    );
    let e108 = diagnostics
        .iter()
        .find(|d| d.code == Some("E108"))
        .expect("expected a diagnostic with code E108");
    assert_eq!(e108.phase, DiagnosticPhase::Semantic);
    assert!(e108.message.contains("output"));
}

#[test]
fn conformance_e109_scalar_vars_is_rejected() {
    // E109: `vars` must be a mapping — scalar value is rejected
    let source = "agent: TestAgent\nsystem: handle requests\nvars: not-a-mapping\n";
    let document = parse_str(source).expect("e109 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for scalar-valued vars"
    );
    let e109 = diagnostics
        .iter()
        .find(|d| d.code == Some("E109"))
        .expect("expected a diagnostic with code E109");
    assert_eq!(e109.phase, DiagnosticPhase::Semantic);
    assert!(e109.message.contains("vars"));
}

#[test]
fn conformance_e110_mapping_value_in_vars_is_rejected() {
    // E110: vars entry must be a scalar value — nested mapping is rejected
    let source = "agent: TestAgent\nsystem: handle requests\nvars:\n  key:\n    nested: value\n";
    let document = parse_str(source).expect("e110 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for mapping-valued vars entry"
    );
    let e110 = diagnostics
        .iter()
        .find(|d| d.code == Some("E110"))
        .expect("expected a diagnostic with code E110");
    assert_eq!(e110.phase, DiagnosticPhase::Semantic);
    assert!(e110.message.contains("vars.key"));
}

// --- Error code conformance: v2 Track B (E103 extension, E111, E112, E113) ---

#[test]
fn conformance_e103_empty_system_scalar_is_rejected() {
    // E103 extension (v2): system and user empty scalars are now rejected,
    // matching the existing empty-scalar check on agent.
    let source = "agent: TestAgent\nsystem: \"\"\n";
    let document = parse_str(source).expect("e103 system source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for empty system scalar"
    );
    let e103 = diagnostics
        .iter()
        .find(|d| d.code == Some("E103") && d.message.contains("system"))
        .expect("expected E103 diagnostic mentioning 'system'");
    assert_eq!(e103.phase, DiagnosticPhase::Semantic);
    assert_eq!(
        e103.to_string(),
        "semantic error at 2:1: [E103] `system` must not be empty"
    );
}

// E111: empty mapping block is a validation error.
// This code is a defensive guard in validate_prompt_field and validate_output_field.
// It is unreachable through parse_str because the parser requires at least one
// entry to produce a Mapping node — `key:` with no indented content produces
// parse error E023 before the validator is reached. A conformance test cannot
// be written without constructing an AST that bypasses parse_str. E111 is
// therefore documented here as parser-enforced and intentionally untestable
// at the conformance level.

// E112: empty sequence is a validation error.
// Same situation as E111: the parser requires at least one item to produce
// a Sequence node. `key:` with no indented items produces parse error E023.
// E112 is a defensive guard unreachable through parse_str.

#[test]
fn conformance_e113_duplicate_tools_items_are_rejected() {
    // E113: duplicate items in tools or constraints are rejected.
    let source = "agent: TestAgent\nsystem: handle requests\ntools:\n  - web_search\n  - calculator\n  - web_search\n";
    let document = parse_str(source).expect("e113 source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected validation errors for duplicate tool item"
    );
    let e113 = diagnostics
        .iter()
        .find(|d| d.code == Some("E113"))
        .expect("expected a diagnostic with code E113");
    assert_eq!(e113.phase, DiagnosticPhase::Semantic);
    assert!(
        e113.message.contains("web_search"),
        "expected diagnostic mentioning the duplicate value, got: {}",
        e113.message
    );
}

#[test]
fn conformance_e113_memory_allows_duplicate_items() {
    // E113 exemption: memory sequences are allowed to have duplicate items
    // (repeating context items across turns is valid).
    let source = "agent: TestAgent\nsystem: handle requests\nmemory:\n  - prior_context\n  - prior_context\n";
    let document = parse_str(source).expect("e113 memory source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors for duplicate memory items, got: {diagnostics}"
    );
}

// --- validate --stdin conformance ---
// These tests exercise the parse_str + validate_document pipeline that
// `validate --stdin` uses. The stdin pipe itself (IO plumbing) is not
// tested here; the goal is coverage of the validation contract for
// inline source strings.

#[test]
fn conformance_validate_stdin_flag_accepts_valid_document() {
    // A fully valid document must produce zero errors — same contract
    // whether the source came from a file or from stdin.
    let source = "agent: StdinAgent\nsystem: handle requests\n";
    let document = parse_str(source).expect("valid stdin source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors for valid stdin document, got: {diagnostics}"
    );
}

#[test]
fn conformance_validate_stdin_flag_rejects_invalid_document() {
    // A document missing the required `system` key must produce E101,
    // matching the output of `validate --stdin` on such input.
    let source = "agent: TestAgent\n";
    let document = parse_str(source).expect("agent-only source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        diagnostics.has_errors(),
        "expected E101 for missing system key"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == Some("E101") && d.message.contains("system")),
        "expected E101 mentioning 'system', got: {diagnostics}"
    );
}

// --- parse --summary conformance ---

#[test]
fn conformance_parse_summary_present_keys_match_document() {
    // Tests the data contract underlying `parse --summary`:
    // the keys reported must match those present in the parsed document,
    // in the canonical top-level key order, and the node count must be
    // positive. The CLI rendering (✓ parsed, keys:, nodes: lines) is
    // verified by manual integration; the underlying document contract
    // is tested here.
    let source = "agent: SummaryTest\nsystem: handle requests\nmemory:\n  - item1\n  - item2\n";
    let document = parse_str(source).expect("summary test source should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no errors, got: {diagnostics}"
    );

    // Keys present in canonical order — matches what `keys:` line would print
    let present_keys: Vec<&str> = TopLevelKey::ordered()
        .into_iter()
        .filter(|&key| document.get(key).is_some())
        .map(|key| key.as_str())
        .collect();
    assert_eq!(present_keys, vec!["agent", "system", "memory"]);

    // All three expected fields are populated in the document
    assert!(document.agent.is_some(), "agent must be present");
    assert!(document.system.is_some(), "system must be present");
    assert!(document.memory.is_some(), "memory must be present");
}
