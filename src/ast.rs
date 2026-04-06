use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Scalar(String),
    Sequence(Vec<Node>),
    Mapping(BTreeMap<String, Node>),
}

impl Node {
    pub fn scalar<T: Into<String>>(value: T) -> Self {
        Self::Scalar(value.into())
    }

    pub fn as_scalar(&self) -> Option<&str> {
        match self {
            Self::Scalar(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&[Node]> {
        match self {
            Self::Sequence(values) => Some(values.as_slice()),
            _ => None,
        }
    }

    pub fn as_mapping(&self) -> Option<&BTreeMap<String, Node>> {
        match self {
            Self::Mapping(values) => Some(values),
            _ => None,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Scalar(_) => "scalar",
            Self::Sequence(_) => "sequence",
            Self::Mapping(_) => "mapping",
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

    pub fn from_str(value: &str) -> Option<Self> {
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
