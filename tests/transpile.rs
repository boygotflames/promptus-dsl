use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use llm_format::cli::transpile::{OutputDestination, TargetArg, TranspileArgs, execute};
use llm_format::provider::Provider;
use llm_format::transpile::{self, Target};
use llm_format::{parse_str, validate_document};

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
            "llm_format_{label}_{}_{}",
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
        provider: Provider::Generic,
        output: None,
        force: false,
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

#[test]
fn shadow_transpile_matches_explicit_generic_provider() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Generic)
        .expect("generic provider shadow transpilation should succeed");

    assert_eq!(
        rendered,
        "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]"
    );
}

#[test]
fn shadow_transpile_matches_explicit_openai_provider() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let rendered = transpile::transpile_with_provider(&document, Target::Shadow, Provider::Openai)
        .expect("openai provider shadow transpilation should succeed");

    assert_eq!(
        rendered,
        "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]"
    );
}

#[test]
fn shadow_transpile_anthropic_produces_v1_xml_output() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_valid_document(source);
    let output =
        transpile::transpile_with_provider(&document, Target::Shadow, Provider::Anthropic)
            .expect("anthropic shadow transpilation should succeed");

    assert!(
        output.contains("<agent>"),
        "expected XML-tag output for anthropic shadow, got: {output}"
    );
    assert!(
        output.starts_with("<agent>DataExtractor</agent>"),
        "expected agent XML tag first, got: {output}"
    );
}

#[test]
fn transpile_execute_returns_stdout_payload_when_no_output_path_is_given() {
    let execution = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::Plain,
        provider: Provider::Generic,
        output: None,
        force: false,
    })
    .expect("stdout transpile execution should succeed");

    assert_eq!(execution.destination, OutputDestination::Stdout);
    assert_eq!(
        execution.rendered,
        "agent: DataExtractor\nsystem:\n  role: financial_analyst\n  output: json\nmemory:\n  - user_history"
    );
}

#[test]
fn transpile_execute_writes_requested_output_file() {
    let temp_dir = TestTempDir::new("transpile_write_file");
    let output_path = temp_dir.path().join("minimal.shadow");

    let execution = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::Shadow,
        provider: Provider::Generic,
        output: Some(output_path.clone()),
        force: false,
    })
    .expect("file transpile execution should succeed");

    assert_eq!(
        execution.destination,
        OutputDestination::File(output_path.clone())
    );
    assert_eq!(
        fs::read_to_string(&output_path).expect("output file should be readable"),
        "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]"
    );
}

#[test]
fn transpile_execute_refuses_to_overwrite_existing_file_without_force() {
    let temp_dir = TestTempDir::new("transpile_no_overwrite");
    let output_path = temp_dir.path().join("existing.txt");
    fs::write(&output_path, "existing content").expect("existing output file should be writable");

    let result = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::JsonIr,
        provider: Provider::Generic,
        output: Some(output_path.clone()),
        force: false,
    });

    let error = result.expect_err("overwrite without force should fail");
    assert!(
        error
            .to_string()
            .contains("refusing to overwrite existing output file"),
        "expected overwrite refusal, got: {error}"
    );
    assert_eq!(
        fs::read_to_string(&output_path).expect("existing file should remain readable"),
        "existing content"
    );
}

#[test]
fn transpile_execute_overwrites_existing_file_with_force() {
    let temp_dir = TestTempDir::new("transpile_force_overwrite");
    let output_path = temp_dir.path().join("existing.txt");
    fs::write(&output_path, "existing content").expect("existing output file should be writable");

    let execution = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::JsonIr,
        provider: Provider::Generic,
        output: Some(output_path.clone()),
        force: true,
    })
    .expect("overwrite with force should succeed");

    assert_eq!(
        execution.destination,
        OutputDestination::File(output_path.clone())
    );
    assert_eq!(
        fs::read_to_string(&output_path).expect("overwritten file should be readable"),
        "{\n  \"agent\": \"DataExtractor\",\n  \"system\": {\n    \"role\": \"financial_analyst\",\n    \"output\": \"json\"\n  },\n  \"memory\": [\n    \"user_history\"\n  ]\n}"
    );
}

#[test]
fn transpile_execute_respects_cli_target_selection() {
    let execution = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::Shadow,
        provider: Provider::Openai,
        output: None,
        force: false,
    })
    .expect("shadow transpile execution should succeed");

    assert_eq!(execution.destination, OutputDestination::Stdout);
    assert_eq!(
        execution.rendered,
        "@a=\"DataExtractor\"\n@s={role=\"financial_analyst\";output=\"json\"}\n@m=[\"user_history\"]"
    );
}

#[test]
fn transpile_execute_reports_missing_input_files() {
    let result = execute(TranspileArgs {
        input: PathBuf::from("examples/does-not-exist.llm"),
        target: TargetArg::Plain,
        provider: Provider::Generic,
        output: None,
        force: false,
    });

    let error = result.expect_err("missing input file should fail");
    assert!(
        error.to_string().contains("failed to read"),
        "expected input read failure, got: {error}"
    );
}

#[test]
fn transpile_execute_reports_missing_output_directories() {
    let temp_dir = TestTempDir::new("transpile_missing_parent");
    let output_path = temp_dir.path().join("missing").join("out.txt");

    let result = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::Plain,
        provider: Provider::Generic,
        output: Some(output_path),
        force: false,
    });

    let error = result.expect_err("missing output directory should fail");
    assert!(
        error
            .to_string()
            .contains("output directory does not exist"),
        "expected missing directory failure, got: {error}"
    );
}

#[test]
fn transpile_execute_anthropic_shadow_succeeds() {
    let result = execute(TranspileArgs {
        input: PathBuf::from("examples/minimal.llm"),
        target: TargetArg::Shadow,
        provider: Provider::Anthropic,
        output: None,
        force: false,
    });

    let execution = result.expect("anthropic shadow transpile should succeed");
    assert!(
        execution.rendered.contains("<agent>"),
        "expected XML-tag shadow output for anthropic, got: {}",
        execution.rendered
    );
}
