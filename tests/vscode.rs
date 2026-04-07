use std::fs;

use regex::Regex;
use serde_json::Value;

fn load_json(path: &str) -> Value {
    let source = fs::read_to_string(path).expect("asset should be readable");
    serde_json::from_str(&source).expect("asset should be valid JSON")
}

fn load_text(path: &str) -> String {
    fs::read_to_string(path).expect("fixture should be readable")
}

fn repository_match<'a>(grammar: &'a Value, key: &str, index: usize) -> &'a str {
    grammar["repository"][key]["patterns"][index]["match"]
        .as_str()
        .expect("repository pattern should provide a match")
}

#[test]
fn vscode_manifest_registers_llm_language_and_grammar() {
    let package = load_json("editors/vscode/package.json");

    assert_eq!(package["name"], "llm-vscode");
    assert_eq!(package["displayName"], ".llm Syntax Support");

    let languages = package["contributes"]["languages"]
        .as_array()
        .expect("languages contribution should be an array");
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0]["id"], "llm");
    assert_eq!(
        languages[0]["configuration"],
        "./language-configuration.json"
    );
    assert!(
        languages[0]["extensions"]
            .as_array()
            .expect("extensions should be an array")
            .iter()
            .any(|value| value == ".llm"),
        "expected .llm extension association"
    );

    let grammars = package["contributes"]["grammars"]
        .as_array()
        .expect("grammars contribution should be an array");
    assert_eq!(grammars.len(), 1);
    assert_eq!(grammars[0]["language"], "llm");
    assert_eq!(grammars[0]["scopeName"], "source.llm");
    assert_eq!(grammars[0]["path"], "./syntaxes/llm.tmLanguage.json");
}

#[test]
fn vscode_language_configuration_declares_comments_and_quote_pairs() {
    let configuration = load_json("editors/vscode/language-configuration.json");

    assert_eq!(configuration["comments"]["lineComment"], "#");

    let auto_closing_pairs = configuration["autoClosingPairs"]
        .as_array()
        .expect("autoClosingPairs should be an array");
    assert!(
        auto_closing_pairs
            .iter()
            .any(|pair| pair["open"] == "\"" && pair["close"] == "\""),
        "expected double-quote auto-closing"
    );
    assert!(
        auto_closing_pairs
            .iter()
            .any(|pair| pair["open"] == "'" && pair["close"] == "'"),
        "expected single-quote auto-closing"
    );
}

#[test]
fn vscode_grammar_patterns_cover_current_fixtures() {
    let grammar = load_json("editors/vscode/syntaxes/llm.tmLanguage.json");
    assert_eq!(grammar["scopeName"], "source.llm");

    let top_level = Regex::new(repository_match(&grammar, "topLevelKeys", 0))
        .expect("top-level key pattern should compile");
    let mapping_key = Regex::new(repository_match(&grammar, "mappingKeys", 0))
        .expect("mapping key pattern should compile");
    let list_marker = Regex::new(repository_match(&grammar, "listMarkers", 0))
        .expect("list marker pattern should compile");
    let double_quoted = Regex::new(repository_match(&grammar, "doubleQuotedStrings", 0))
        .expect("double-quoted string pattern should compile");
    let single_quoted = Regex::new(repository_match(&grammar, "singleQuotedStrings", 0))
        .expect("single-quoted string pattern should compile");
    let comment = Regex::new(repository_match(&grammar, "comments", 0))
        .expect("comment pattern should compile");
    let mapping_scalar = Regex::new(repository_match(&grammar, "bareScalars", 0))
        .expect("mapping scalar pattern should compile");
    let list_scalar = Regex::new(repository_match(&grammar, "bareScalars", 1))
        .expect("list scalar pattern should compile");

    let minimal = load_text("examples/minimal.llm");
    let quoted = load_text("examples/quoted.llm");
    let messy = load_text("examples/noncanonical/messy.llm");
    let highlighting = load_text("editors/vscode/fixtures/highlighting-sample.llm");

    assert!(
        minimal.lines().any(|line| top_level.is_match(line)),
        "expected top-level grammar matches in minimal fixture"
    );
    assert!(
        quoted.lines().any(|line| top_level.is_match(line)),
        "expected top-level grammar matches in quoted fixture"
    );
    assert!(
        minimal.lines().any(|line| mapping_key.is_match(line)),
        "expected mapping-key matches in minimal fixture"
    );
    assert!(
        quoted.lines().any(|line| mapping_key.is_match(line)),
        "expected mapping-key matches in quoted fixture"
    );
    assert!(
        minimal.lines().any(|line| list_marker.is_match(line)),
        "expected list-marker matches in minimal fixture"
    );
    assert!(
        minimal.lines().any(|line| mapping_scalar.is_match(line)),
        "expected bare mapping scalars in minimal fixture"
    );
    assert!(
        minimal.lines().any(|line| list_scalar.is_match(line)),
        "expected bare list scalars in minimal fixture"
    );
    assert!(
        quoted.lines().any(|line| double_quoted.is_match(line)),
        "expected double-quoted string matches in quoted fixture"
    );
    assert!(
        quoted.lines().any(|line| single_quoted.is_match(line)),
        "expected single-quoted string matches in quoted fixture"
    );
    assert!(
        messy.lines().any(|line| single_quoted.is_match(line)),
        "expected single-quoted string matches in messy fixture"
    );
    assert!(
        highlighting.lines().any(|line| comment.is_match(line)),
        "expected comment matches in highlighting fixture"
    );
}

#[test]
fn vscode_readme_documents_manual_local_verification() {
    let readme = load_text("editors/vscode/README.md");

    assert!(
        readme.contains("Run .llm Syntax Extension"),
        "expected a named manual launch path"
    );
    assert!(
        readme.contains("examples/minimal.llm"),
        "expected a concrete fixture path in the manual verification steps"
    );
    assert!(
        readme.contains("does not call into the Rust parser or validator"),
        "expected documentation of the extension boundary"
    );
}
