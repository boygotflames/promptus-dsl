use llm_format::{Node, Span, TopLevelKey, parse_str};

#[test]
fn parses_minimal_example_into_a_stable_ast() {
    let document =
        parse_str(include_str!("../examples/minimal.llm")).expect("minimal example should parse");

    assert_eq!(
        format!("{document:#?}"),
        r#"Document {
    agent: Some(
        Scalar {
            value: "DataExtractor",
            span: Span {
                line: 1,
                column: 1,
            },
        },
    ),
    system: Some(
        Mapping {
            entries: [
                MappingEntry {
                    key: "role",
                    value: Scalar {
                        value: "financial_analyst",
                        span: Span {
                            line: 3,
                            column: 3,
                        },
                    },
                    span: Span {
                        line: 3,
                        column: 3,
                    },
                },
                MappingEntry {
                    key: "output",
                    value: Scalar {
                        value: "json",
                        span: Span {
                            line: 4,
                            column: 3,
                        },
                    },
                    span: Span {
                        line: 4,
                        column: 3,
                    },
                },
            ],
            span: Span {
                line: 2,
                column: 1,
            },
        },
    ),
    user: None,
    memory: Some(
        Sequence {
            values: [
                Scalar {
                    value: "user_history",
                    span: Span {
                        line: 6,
                        column: 3,
                    },
                },
            ],
            span: Span {
                line: 5,
                column: 1,
            },
        },
    ),
    tools: None,
    output: None,
    constraints: None,
    vars: None,
}"#
    );
}

#[test]
fn parses_quoted_scalars_and_nested_maps() {
    let document =
        parse_str(include_str!("../examples/quoted.llm")).expect("quoted example should parse");

    assert_eq!(
        document.get(TopLevelKey::Agent),
        Some(&Node::scalar("Data Extractor"))
    );
    assert_eq!(
        document.get(TopLevelKey::User).and_then(Node::as_scalar),
        Some("Summarize \"Q1\" results")
    );

    let vars = document
        .get(TopLevelKey::Vars)
        .and_then(Node::as_mapping)
        .expect("vars should be a mapping");
    let company = vars
        .iter()
        .find(|entry| entry.key == "company")
        .map(|entry| &entry.value);

    assert_eq!(company, Some(&Node::scalar("Acme Corp")));
    assert_eq!(
        document.get(TopLevelKey::System).map(Node::span),
        Some(Span::new(2, 1))
    );
}
