use llm_format::parse_str;

#[test]
fn reports_unknown_top_level_keys() {
    let diagnostics = parse_str(include_str!("../examples/invalid/unknown-key.llm"))
        .expect_err("unknown top-level key should fail");

    assert_eq!(
        diagnostics.to_string(),
        "error at 2:1: unknown top-level key `persona`"
    );
}

#[test]
fn reports_mismatched_indentation() {
    let diagnostics = parse_str(include_str!("../examples/invalid/bad-indentation.llm"))
        .expect_err("bad indentation should fail");

    assert_eq!(
        diagnostics.to_string(),
        "error at 3:5: nested blocks must be indented by exactly 2 spaces"
    );
}

#[test]
fn reports_missing_colons() {
    let diagnostics = parse_str(include_str!("../examples/invalid/missing-colon.llm"))
        .expect_err("missing colon should fail");

    assert_eq!(
        diagnostics.to_string(),
        "error at 1:6: expected `:` after mapping key"
    );
}

#[test]
fn reports_unterminated_quoted_scalars() {
    let diagnostics = parse_str(include_str!("../examples/invalid/unterminated-string.llm"))
        .expect_err("unterminated string should fail");

    assert_eq!(
        diagnostics.to_string(),
        "error at 1:8: unterminated quoted scalar"
    );
}
