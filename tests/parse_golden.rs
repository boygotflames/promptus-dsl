use llm_format::{Node, TopLevelKey, parse_str};

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

    assert_eq!(system.get("role"), Some(&Node::scalar("financial_analyst")));
    assert_eq!(system.get("output"), Some(&Node::scalar("json")));
}
