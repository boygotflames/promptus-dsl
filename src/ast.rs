use crate::diagnostics::Span;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    /// Composition directive: resolved before validation/transpilation.
    /// Never appears in transpiled output.
    pub include: Option<Node>,
    pub agent: Option<Node>,
    pub system: Option<Node>,
    pub user: Option<Node>,
    pub memory: Option<Node>,
    pub tools: Option<Node>,
    pub output: Option<Node>,
    pub constraints: Option<Node>,
    pub vars: Option<Node>,
}

impl Document {
    pub fn set(&mut self, key: TopLevelKey, value: Node) -> Option<Node> {
        match key {
            TopLevelKey::Agent => self.agent.replace(value),
            TopLevelKey::System => self.system.replace(value),
            TopLevelKey::User => self.user.replace(value),
            TopLevelKey::Memory => self.memory.replace(value),
            TopLevelKey::Tools => self.tools.replace(value),
            TopLevelKey::Output => self.output.replace(value),
            TopLevelKey::Constraints => self.constraints.replace(value),
            TopLevelKey::Vars => self.vars.replace(value),
        }
    }

    pub fn get(&self, key: TopLevelKey) -> Option<&Node> {
        match key {
            TopLevelKey::Agent => self.agent.as_ref(),
            TopLevelKey::System => self.system.as_ref(),
            TopLevelKey::User => self.user.as_ref(),
            TopLevelKey::Memory => self.memory.as_ref(),
            TopLevelKey::Tools => self.tools.as_ref(),
            TopLevelKey::Output => self.output.as_ref(),
            TopLevelKey::Constraints => self.constraints.as_ref(),
            TopLevelKey::Vars => self.vars.as_ref(),
        }
    }

    pub fn ordered_entries(&self) -> Vec<(TopLevelKey, &Node)> {
        TopLevelKey::ordered()
            .into_iter()
            .filter_map(|key| self.get(key).map(|value| (key, value)))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct MappingEntry {
    pub key: String,
    pub value: Node,
    pub span: Span,
}

impl MappingEntry {
    pub fn new<T: Into<String>>(key: T, value: Node, span: Span) -> Self {
        Self {
            key: key.into(),
            value,
            span,
        }
    }
}

impl PartialEq for MappingEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl Eq for MappingEntry {}

#[derive(Clone, Debug)]
pub enum Node {
    Scalar {
        value: String,
        span: Span,
    },
    Sequence {
        values: Vec<Node>,
        span: Span,
    },
    Mapping {
        entries: Vec<MappingEntry>,
        span: Span,
    },
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar { value: left, .. }, Self::Scalar { value: right, .. }) => left == right,
            (Self::Sequence { values: left, .. }, Self::Sequence { values: right, .. }) => {
                left == right
            }
            (Self::Mapping { entries: left, .. }, Self::Mapping { entries: right, .. }) => {
                left == right
            }
            _ => false,
        }
    }
}

impl Eq for Node {}

impl Node {
    pub fn scalar<T: Into<String>>(value: T) -> Self {
        Self::scalar_at(value, Span::new(0, 0))
    }

    pub fn scalar_at<T: Into<String>>(value: T, span: Span) -> Self {
        Self::Scalar {
            value: value.into(),
            span,
        }
    }

    pub fn sequence(items: Vec<Node>) -> Self {
        Self::sequence_at(items, Span::new(0, 0))
    }

    pub fn sequence_at(items: Vec<Node>, span: Span) -> Self {
        Self::Sequence {
            values: items,
            span,
        }
    }

    pub fn mapping(entries: Vec<MappingEntry>) -> Self {
        Self::mapping_at(entries, Span::new(0, 0))
    }

    pub fn mapping_at(entries: Vec<MappingEntry>, span: Span) -> Self {
        Self::Mapping { entries, span }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Scalar { span, .. }
            | Self::Sequence { span, .. }
            | Self::Mapping { span, .. } => *span,
        }
    }

    pub fn as_scalar(&self) -> Option<&str> {
        match self {
            Self::Scalar { value, .. } => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&[Node]> {
        match self {
            Self::Sequence { values, .. } => Some(values.as_slice()),
            _ => None,
        }
    }

    pub fn as_mapping(&self) -> Option<&[MappingEntry]> {
        match self {
            Self::Mapping { entries, .. } => Some(entries.as_slice()),
            _ => None,
        }
    }

    pub fn mapping_get(&self, key: &str) -> Option<&Node> {
        self.as_mapping()?
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Scalar { .. } => "scalar",
            Self::Sequence { .. } => "sequence",
            Self::Mapping { .. } => "mapping",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TopLevelKey {
    Agent,
    System,
    User,
    Memory,
    Tools,
    Output,
    Constraints,
    Vars,
}

impl TopLevelKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::System => "system",
            Self::User => "user",
            Self::Memory => "memory",
            Self::Tools => "tools",
            Self::Output => "output",
            Self::Constraints => "constraints",
            Self::Vars => "vars",
        }
    }

    pub fn from_keyword(value: &str) -> Option<Self> {
        match value {
            "agent" => Some(Self::Agent),
            "system" => Some(Self::System),
            "user" => Some(Self::User),
            "memory" => Some(Self::Memory),
            "tools" => Some(Self::Tools),
            "output" => Some(Self::Output),
            "constraints" => Some(Self::Constraints),
            "vars" => Some(Self::Vars),
            _ => None,
        }
    }

    pub fn ordered() -> [Self; 8] {
        [
            Self::Agent,
            Self::System,
            Self::User,
            Self::Memory,
            Self::Tools,
            Self::Output,
            Self::Constraints,
            Self::Vars,
        ]
    }
}
