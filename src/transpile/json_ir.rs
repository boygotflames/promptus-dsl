use crate::ast::{Document, Node};

use super::quote;

pub fn transpile(document: &Document) -> String {
    let mut lines = Vec::new();
    lines.push("{".to_owned());

    let entries = document.ordered_entries();
    for (index, (key, value)) in entries.iter().enumerate() {
        let suffix = if index + 1 == entries.len() { "" } else { "," };
        lines.push(format!(
            "  {}: {}{}",
            quote(key.as_str()),
            render_node(value, 4),
            suffix
        ));
    }

    lines.push("}".to_owned());
    lines.join("\n")
}

fn render_node(value: &Node, indent: usize) -> String {
    match value {
        Node::Scalar { value: scalar, .. } => quote(scalar),
        Node::Sequence { values: items, .. } => {
            if items.is_empty() {
                return "[]".to_owned();
            }

            let mut lines = Vec::new();
            lines.push("[".to_owned());
            for (index, item) in items.iter().enumerate() {
                let suffix = if index + 1 == items.len() { "" } else { "," };
                lines.push(format!(
                    "{}{}{}",
                    " ".repeat(indent),
                    render_node(item, indent + 2),
                    suffix
                ));
            }
            lines.push(format!("{}]", " ".repeat(indent.saturating_sub(2))));
            lines.join("\n")
        }
        Node::Mapping { entries, .. } => {
            if entries.is_empty() {
                return "{}".to_owned();
            }

            let mut lines = Vec::new();
            lines.push("{".to_owned());

            for (index, entry) in entries.iter().enumerate() {
                let suffix = if index + 1 == entries.len() { "" } else { "," };
                lines.push(format!(
                    "{}{}: {}{}",
                    " ".repeat(indent),
                    quote(&entry.key),
                    render_node(&entry.value, indent + 2),
                    suffix
                ));
            }

            lines.push(format!("{}{}", " ".repeat(indent.saturating_sub(2)), "}"));
            lines.join("\n")
        }
    }
}
