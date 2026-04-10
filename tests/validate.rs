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
        "semantic error at 5:3: [E102] duplicate key `role` in `system`"
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

#[test]
fn empty_agent_scalar_is_rejected() {
    let source = include_str!("../examples/invalid/empty-agent.llm");
    let document = parse_str(source).expect("empty-agent fixture should parse");
    let diagnostics = validate_document(&document);

    assert!(
        diagnostics.has_errors(),
        "expected validation errors for empty agent scalar"
    );
    let diagnostic = diagnostics
        .iter()
        .find(|d| d.code == Some("E103"))
        .expect("expected a diagnostic with code E103");
    assert!(
        diagnostic.message.contains("agent") || diagnostic.message.contains("must not be empty"),
        "expected diagnostic message to mention agent or emptiness, got: {}",
        diagnostic.message
    );
    assert_eq!(diagnostic.phase, DiagnosticPhase::Semantic);
}

#[test]
fn missing_agent_key_is_rejected() {
    let source = include_str!("../examples/invalid/missing-agent.llm");
    let document = parse_str(source).expect("missing-agent fixture should parse");
    let diagnostics = validate_document(&document);

    assert!(
        diagnostics.has_errors(),
        "expected validation errors for missing agent key"
    );
    let diagnostic = diagnostics
        .iter()
        .find(|d| d.message.contains("agent"))
        .expect("expected a diagnostic mentioning 'agent'");
    assert_eq!(diagnostic.phase, DiagnosticPhase::Semantic);
    assert_eq!(
        diagnostic.to_string(),
        "semantic error at 1:1: [E101] missing required key: `agent`"
    );
}
