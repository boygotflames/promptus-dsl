use llm_format::transpile::{self, Target};
use llm_format::{parse_str, validate_document};

#[test]
fn plain_transpile_preserves_surface_shape() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_str(source).expect("minimal example should parse");
    let diagnostics = validate_document(&document);
    assert!(
        !diagnostics.has_errors(),
        "expected no validation errors, got: {diagnostics}"
    );

    let rendered = transpile::transpile(&document, Target::Plain);
    assert!(rendered.contains("agent: DataExtractor"));
    assert!(rendered.contains("memory:\n  - user_history"));
}

#[test]
fn shadow_transpile_flattens_nested_paths() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_str(source).expect("minimal example should parse");
    let rendered = transpile::transpile(&document, Target::Shadow);

    assert!(rendered.contains("system.role = \"financial_analyst\""));
    assert!(rendered.contains("memory[0] = \"user_history\""));
}
