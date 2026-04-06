use llm_format::{DiagnosticPhase, Span, parse_str, validate_document};

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
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("`vars`"))
        .expect("expected a vars diagnostic");
    assert_eq!(diagnostic.span, Some(Span::new(3, 1)));
}

#[test]
fn duplicate_nested_keys_are_rejected_by_validation() {
    let source = r#"
agent: DataExtractor
system:
  role: first
  role: second
"#;

    let document = parse_str(source).expect("duplicate mapping keys should parse");
    let diagnostics = validate_document(&document);

    assert!(diagnostics.has_errors());
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("duplicate key `role`"))
        .expect("expected a duplicate-key diagnostic");
    assert_eq!(diagnostic.span, Some(Span::new(5, 3)));
    assert_eq!(diagnostic.phase, DiagnosticPhase::Semantic);
    assert_eq!(
        diagnostic.to_string(),
        "semantic error at 5:3: duplicate key `role` in `system`"
    );
}

#[test]
fn same_key_name_in_different_map_scopes_is_allowed() {
    let source = r#"
agent: DataExtractor
system:
  role: summarizer
  nested:
    role: helper
"#;

    let document = parse_str(source).expect("fixture should parse");
    let diagnostics = validate_document(&document);

    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors, got: {diagnostics}"
    );
}
