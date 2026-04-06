use llm_format::{Node, Span, TopLevelKey, parse_str};

#[test]
fn parses_minimal_example() {
    let source = include_str!("../examples/minimal.llm");
    let document = parse_str(source).expect("minimal example should parse");

    assert_eq!(
        document.get(TopLevelKey::Agent),
        Some(&Node::scalar("DataExtractor"))
    );

    let system = document
        .get(TopLevelKey::System)
        .and_then(Node::as_mapping)
        .expect("system should be a mapping");

    let role = system
        .iter()
        .find(|entry| entry.key == "role")
        .map(|entry| &entry.value);
    let output = system
        .iter()
        .find(|entry| entry.key == "output")
        .map(|entry| &entry.value);

    assert_eq!(role, Some(&Node::scalar("financial_analyst")));
    assert_eq!(output, Some(&Node::scalar("json")));

    assert_eq!(
        document.get(TopLevelKey::Agent).map(Node::span),
        Some(Span::new(1, 1))
    );
    assert_eq!(role.map(Node::span), Some(Span::new(3, 3)));
    assert_eq!(
        document
            .get(TopLevelKey::Memory)
            .and_then(Node::as_sequence)
            .and_then(|items| items.first())
            .map(Node::span),
        Some(Span::new(6, 3))
    );
}
