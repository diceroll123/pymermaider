/// Represents visibility of class members (public, private, protected)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
    /// Reserved for future use - Python doesn't have true protected visibility
    #[allow(dead_code)]
    Protected,
}

/// Represents a class attribute/field
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub name: String,
    pub type_annotation: String,
    pub visibility: Visibility,
}

/// Represents a method parameter
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: String,
    pub return_type: Option<String>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_async: bool,
    pub decorators: Vec<String>,
}

/// Type of class (regular, abstract, interface/protocol, enum, dataclass)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassType {
    Regular,
    Abstract,
    Interface,
    Enumeration,
    Dataclass,
    Final,
}

/// Represents a class node in the diagram
#[derive(Debug, Clone)]
pub struct ClassNode {
    pub name: String,
    pub type_params: Option<String>,
    pub class_type: ClassType,
    pub attributes: Vec<Attribute>,
    pub methods: Vec<MethodSignature>,
}

/// Type of relationship between classes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationType {
    Inheritance,    // Solid line: --|>
    Implementation, // Dotted line: ..|> (for interfaces/abstracts)
}

/// Represents an inheritance or implementation relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelationshipEdge {
    pub from: String,
    pub to: String,
    pub relation_type: RelationType,
}

/// Represents a composition relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompositionEdge {
    pub container: String,
    pub contained: String,
}

/// Class diagram direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiagramDirection {
    /// Top to bottom (default)
    #[default]
    TB,
    /// Bottom to top
    BT,
    /// Left to right
    LR,
    /// Right to left
    RL,
}

impl std::fmt::Display for DiagramDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagramDirection::TB => write!(f, "TB"),
            DiagramDirection::BT => write!(f, "BT"),
            DiagramDirection::LR => write!(f, "LR"),
            DiagramDirection::RL => write!(f, "RL"),
        }
    }
}

impl std::str::FromStr for DiagramDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TB" => Ok(Self::TB),
            "BT" => Ok(Self::BT),
            "LR" => Ok(Self::LR),
            "RL" => Ok(Self::RL),
            _ => Err(format!(
                "invalid direction: {s} (expected TB, BT, LR, or RL)"
            )),
        }
    }
}

/// The complete diagram structure
#[derive(Debug, Clone, Default)]
pub struct Diagram {
    pub classes: Vec<ClassNode>,
    pub relationships: Vec<RelationshipEdge>,
    pub compositions: Vec<CompositionEdge>,
}

impl Diagram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.relationships.is_empty() && self.compositions.is_empty()
    }

    pub fn add_class(&mut self, class: ClassNode) {
        self.classes.push(class);
    }

    pub fn add_relationship(&mut self, relationship: RelationshipEdge) {
        self.relationships.push(relationship);
    }

    pub fn add_composition(&mut self, composition: CompositionEdge) {
        self.compositions.push(composition);
    }

    pub fn is_abstract_or_interface(&self, name: &str) -> bool {
        self.classes.iter().any(|c| {
            c.name == name && matches!(c.class_type, ClassType::Abstract | ClassType::Interface)
        })
    }

    /// Return classes in a deterministic topological order based on relationships and compositions,
    /// without mutating the diagram. Classes are de-duplicated by name (first occurrence wins).
    pub fn classes_topologically_sorted_unique(&self) -> Vec<&ClassNode> {
        use std::collections::{HashMap, HashSet};

        // Build a name -> first ClassNode map (stable de-dupe).
        let mut class_map: HashMap<&str, &ClassNode> = HashMap::new();
        for class in &self.classes {
            class_map.entry(class.name.as_str()).or_insert(class);
        }

        // Build dependency graph from relationships and compositions.
        let mut dependencies: HashMap<&str, HashSet<&str>> = HashMap::new();

        for relationship in &self.relationships {
            dependencies
                .entry(relationship.from.as_str())
                .or_default()
                .insert(relationship.to.as_str());
        }

        for composition in &self.compositions {
            dependencies
                .entry(composition.container.as_str())
                .or_default()
                .insert(composition.contained.as_str());
        }

        // Topological sort with deterministic iteration.
        fn visit<'a>(
            name: &'a str,
            dependencies: &HashMap<&'a str, HashSet<&'a str>>,
            visited: &mut HashSet<&'a str>,
            sorted: &mut Vec<&'a str>,
        ) {
            if visited.contains(name) {
                return;
            }
            visited.insert(name);

            if let Some(deps) = dependencies.get(name) {
                let mut sorted_deps: Vec<_> = deps.iter().copied().collect();
                sorted_deps.sort();
                for dep in sorted_deps {
                    visit(dep, dependencies, visited, sorted);
                }
            }

            sorted.push(name);
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut sorted_names: Vec<&str> = Vec::new();

        let mut class_names: Vec<&str> = class_map.keys().copied().collect();
        class_names.sort();
        for name in class_names {
            visit(name, &dependencies, &mut visited, &mut sorted_names);
        }

        sorted_names
            .into_iter()
            .filter_map(|name| class_map.get(name).copied())
            .collect()
    }
}
