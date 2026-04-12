//! vars expansion: substitute `{var_name}` references in scalar values.
//!
//! Expansion occurs at transpile time across the `plain`, `json-ir`, and `shadow`
//! targets. The `fmt` formatter deliberately does NOT expand — it preserves `{var}`
//! references verbatim so the source file remains editable.
//!
//! Expansion is non-recursive: substituted values are not scanned for further
//! `{var}` references. The `vars` block itself is never expanded.

use std::collections::HashMap;

use crate::ast::{Document, MappingEntry, Node};

// ── Public helpers ────────────────────────────────────────────────────────────

/// Build a `{name → value}` map from the document's `vars` block.
/// Returns an empty map when no `vars` block is present.
pub fn build_vars_map(document: &Document) -> HashMap<String, String> {
    let Some(vars_node) = document.vars.as_ref() else {
        return HashMap::new();
    };
    let Some(entries) = vars_node.as_mapping() else {
        return HashMap::new();
    };
    entries
        .iter()
        .filter_map(|e| e.value.as_scalar().map(|v| (e.key.clone(), v.to_owned())))
        .collect()
}

/// Expand all `{var_name}` references in `value` using `vars`.
/// References to unknown keys are passed through verbatim.
/// Expansion is non-recursive.
pub fn expand(value: &str, vars: &HashMap<String, String>) -> String {
    if vars.is_empty() || !value.contains('{') {
        return value.to_owned();
    }

    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut name = String::new();
            let mut closed = false;
            for inner in chars.by_ref() {
                if inner == '}' {
                    closed = true;
                    break;
                }
                name.push(inner);
            }
            if closed && !name.is_empty() && is_valid_key(&name) {
                if let Some(val) = vars.get(&name) {
                    result.push_str(val);
                } else {
                    // Unknown reference — pass through verbatim
                    result.push('{');
                    result.push_str(&name);
                    result.push('}');
                }
            } else {
                // Not a valid reference — pass through verbatim
                result.push('{');
                result.push_str(&name);
                if closed {
                    result.push('}');
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Extract all `{var_name}` reference names from `value`.
/// Returns only names that match the key grammar.
/// Duplicate names are returned as-is (not deduplicated).
pub fn extract_references(value: &str) -> Vec<String> {
    if !value.contains('{') {
        return Vec::new();
    }

    let mut refs = Vec::new();
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut name = String::new();
            let mut closed = false;
            for inner in chars.by_ref() {
                if inner == '}' {
                    closed = true;
                    break;
                }
                name.push(inner);
            }
            if closed && !name.is_empty() && is_valid_key(&name) {
                refs.push(name);
            }
        }
    }

    refs
}

/// Return a copy of `document` with all `{var_name}` references substituted
/// using the document's own `vars` block.
///
/// The `vars` block itself is never modified — its values are the source of
/// truth and are not expanded. The caller must not expand the returned document
/// again (expansion is non-recursive by design).
///
/// If `vars` is absent or empty, returns a structurally identical document
/// (all nodes are reconstructed; spans are preserved).
pub fn expand_document(document: &Document) -> Document {
    let vars_map = build_vars_map(document);
    Document {
        agent: document
            .agent
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        system: document
            .system
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        user: document
            .user
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        memory: document
            .memory
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        tools: document
            .tools
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        output: document
            .output
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        constraints: document
            .constraints
            .as_ref()
            .map(|n| expand_node_tree(n, &vars_map)),
        // vars is the expansion source — never expand its own values
        vars: document.vars.as_ref().map(clone_node),
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn expand_node_tree(node: &Node, vars_map: &HashMap<String, String>) -> Node {
    match node {
        Node::Scalar { value, span } => Node::Scalar {
            value: expand(value, vars_map),
            span: *span,
        },
        Node::Mapping { entries, span } => Node::Mapping {
            entries: entries
                .iter()
                .map(|e| {
                    MappingEntry::new(e.key.clone(), expand_node_tree(&e.value, vars_map), e.span)
                })
                .collect(),
            span: *span,
        },
        Node::Sequence { values, span } => Node::Sequence {
            values: values
                .iter()
                .map(|v| expand_node_tree(v, vars_map))
                .collect(),
            span: *span,
        },
    }
}

/// Deep-clone a node without expanding any values.
fn clone_node(node: &Node) -> Node {
    match node {
        Node::Scalar { value, span } => Node::Scalar {
            value: value.clone(),
            span: *span,
        },
        Node::Mapping { entries, span } => Node::Mapping {
            entries: entries
                .iter()
                .map(|e| MappingEntry::new(e.key.clone(), clone_node(&e.value), e.span))
                .collect(),
            span: *span,
        },
        Node::Sequence { values, span } => Node::Sequence {
            values: values.iter().map(clone_node).collect(),
            span: *span,
        },
    }
}

fn is_valid_key(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn expands_known_reference() {
        let v = vars(&[("region", "apac")]);
        assert_eq!(expand("Hello {region}", &v), "Hello apac");
    }

    #[test]
    fn passes_through_unknown_reference() {
        let v = vars(&[("region", "apac")]);
        assert_eq!(expand("Hello {unknown}", &v), "Hello {unknown}");
    }

    #[test]
    fn no_vars_returns_value_unchanged() {
        let v = vars(&[]);
        assert_eq!(expand("Hello {region}", &v), "Hello {region}");
    }

    #[test]
    fn non_recursive_expansion() {
        let v = vars(&[("a", "{b}"), ("b", "final")]);
        assert_eq!(expand("{a}", &v), "{b}");
    }

    #[test]
    fn no_braces_returns_value_unchanged() {
        let v = vars(&[("region", "apac")]);
        assert_eq!(expand("no braces here", &v), "no braces here");
    }

    #[test]
    fn extract_references_finds_names() {
        let refs = extract_references("Connect to {source_table} in {region}");
        assert_eq!(refs, vec!["source_table", "region"]);
    }

    #[test]
    fn extract_references_empty_when_no_braces() {
        let refs = extract_references("plain scalar");
        assert!(refs.is_empty());
    }

    #[test]
    fn is_valid_key_accepts_valid_names() {
        assert!(is_valid_key("region"));
        assert!(is_valid_key("source_table"));
        assert!(is_valid_key("my-var"));
        assert!(is_valid_key("_private"));
    }

    #[test]
    fn is_valid_key_rejects_invalid_names() {
        assert!(!is_valid_key(""));
        assert!(!is_valid_key("123bad"));
        assert!(!is_valid_key("has space"));
    }
}
