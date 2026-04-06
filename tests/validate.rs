use llm_format::{parse_str, validate_document};

#[test]
fn valid_example_passes_validation() {
    let source = include_str!("../examples/extractor.llm");
    let document = parse_str(source).expect("extractor example should parse");
    let diagnostics = validate_document(&document);

    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors, got: {diagnostics}"
    );
}

#[test]
fn vars_must_be_a_mapping() {
    let source = r#"
agent: DataExtractor
vars:
  - not_allowed
"#;

    let document = parse_str(source).expect("fixture should parse");
    let diagnostics = validate_document(&document);

    assert!(diagnostics.has_errors());
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("`vars`")),
        "expected a vars diagnostic, got: {diagnostics}"
    );
}
