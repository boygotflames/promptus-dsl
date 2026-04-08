use anyhow::Result;

use crate::ast::{Document, Node, TopLevelKey};
use crate::provider::{Provider, ShadowProfile};

use super::{Emitter, quote};

pub struct ShadowEmitter;

#[derive(Clone, Copy)]
struct ShadowTopLevelMarker {
    key: TopLevelKey,
    marker: &'static str,
}

#[derive(Clone, Copy)]
struct ShadowSyntax {
    assignment: &'static str,
    mapping_open: &'static str,
    mapping_close: &'static str,
    sequence_open: &'static str,
    sequence_close: &'static str,
    entry_separator: &'static str,
    value_separator: &'static str,
}

const TOP_LEVEL_MARKERS: [ShadowTopLevelMarker; 8] = [
    ShadowTopLevelMarker {
        key: TopLevelKey::Agent,
        marker: "@a",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::System,
        marker: "@s",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::User,
        marker: "@u",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::Memory,
        marker: "@m",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::Tools,
        marker: "@t",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::Output,
        marker: "@o",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::Constraints,
        marker: "@c",
    },
    ShadowTopLevelMarker {
        key: TopLevelKey::Vars,
        marker: "@v",
    },
];

const SHADOW_SYNTAX: ShadowSyntax = ShadowSyntax {
    assignment: "=",
    mapping_open: "{",
    mapping_close: "}",
    sequence_open: "[",
    sequence_close: "]",
    entry_separator: ";",
    value_separator: ",",
};

impl Emitter for ShadowEmitter {
    fn emit(&self, document: &Document) -> String {
        emit_with_provider(document, Provider::Generic)
            .expect("generic provider must support the provisional v0 shadow profile")
    }
}

pub fn emit_with_provider(document: &Document, provider: Provider) -> Result<String> {
    let shadow_profile = provider.profile().shadow_profile()?;
    Ok(render_document(document, shadow_profile))
}

fn render_document(document: &Document, shadow_profile: ShadowProfile) -> String {
    match shadow_profile {
        ShadowProfile::ProvisionalV0 => render_v0_document(document),
    }
}

fn render_v0_document(document: &Document) -> String {
    let mut lines = Vec::new();

    for (key, value) in document.ordered_entries() {
        lines.push(format!(
            "{}{}{}",
            marker_for(key),
            SHADOW_SYNTAX.assignment,
            render_node(value)
        ));
    }

    lines.join("\n")
}

fn marker_for(key: TopLevelKey) -> &'static str {
    TOP_LEVEL_MARKERS
        .iter()
        .find(|marker| marker.key == key)
        .map(|marker| marker.marker)
        .expect("every top-level key must have a shadow marker")
}

fn render_node(value: &Node) -> String {
    match value {
        Node::Scalar { value: scalar, .. } => quote(scalar),
        Node::Mapping { entries, .. } => render_mapping(entries),
        Node::Sequence { values, .. } => render_sequence(values),
    }
}

fn render_mapping(entries: &[crate::ast::MappingEntry]) -> String {
    let mut rendered = String::from(SHADOW_SYNTAX.mapping_open);

    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            rendered.push_str(SHADOW_SYNTAX.entry_separator);
        }

        rendered.push_str(&entry.key);
        rendered.push_str(SHADOW_SYNTAX.assignment);
        rendered.push_str(&render_node(&entry.value));
    }

    rendered.push_str(SHADOW_SYNTAX.mapping_close);
    rendered
}

fn render_sequence(values: &[Node]) -> String {
    let mut rendered = String::from(SHADOW_SYNTAX.sequence_open);

    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            rendered.push_str(SHADOW_SYNTAX.value_separator);
        }

        rendered.push_str(&render_node(value));
    }

    rendered.push_str(SHADOW_SYNTAX.sequence_close);
    rendered
}
