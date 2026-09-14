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

/// Whether a composition is strong (owned) or weak (referenced/optional)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositionKind {
    Composition, // *-- strong ownership
    Aggregation, // o-- weak/optional reference
}

/// Represents a composition relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompositionEdge {
    pub container: String,
    pub contained: String,
    pub kind: CompositionKind,
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
            Self::TB => write!(f, "TB"),
            Self::BT => write!(f, "BT"),
            Self::LR => write!(f, "LR"),
            Self::RL => write!(f, "RL"),
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
    abstract_or_interface_index: std::collections::HashMap<String, bool>,
}

impl Diagram {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.relationships.is_empty() && self.compositions.is_empty()
    }

    pub fn add_class(&mut self, class: ClassNode) {
        let is_abstract_or_interface =
            matches!(class.class_type, ClassType::Abstract | ClassType::Interface);
        let entry = self
            .abstract_or_interface_index
            .entry(class.name.clone())
            .or_insert(false);
        *entry = *entry || is_abstract_or_interface;
        self.classes.push(class);
    }

    pub fn add_relationship(&mut self, relationship: RelationshipEdge) {
        self.relationships.push(relationship);
    }

    pub fn add_composition(&mut self, composition: CompositionEdge) {
        self.compositions.push(composition);
    }

    #[must_use]
    pub fn is_abstract_or_interface(&self, name: &str) -> bool {
        self.abstract_or_interface_index
            .get(name)
            .copied()
            .unwrap_or(false)
    }

    pub fn extend(&mut self, other: Diagram) {
        self.classes.extend(other.classes);
        self.relationships.extend(other.relationships);
        self.compositions.extend(other.compositions);
        for (name, other_flag) in other.abstract_or_interface_index {
            self.abstract_or_interface_index
                .entry(name)
                .and_modify(|v| *v = *v || other_flag)
                .or_insert(other_flag);
        }
    }

    /// Return classes in a deterministic topological order based on relationships and compositions,
    /// without mutating the diagram. Classes are de-duplicated by name (first occurrence wins).
    #[must_use]
    pub fn classes_topologically_sorted_unique(&self) -> Vec<&ClassNode> {
        use std::collections::{HashMap, HashSet};

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
                sorted_deps.sort_unstable();
                for dep in sorted_deps {
                    visit(dep, dependencies, visited, sorted);
                }
            }

            sorted.push(name);
        }

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

        let mut visited: HashSet<&str> = HashSet::new();
        let mut sorted_names: Vec<&str> = Vec::new();

        let mut class_names: Vec<&str> = class_map.keys().copied().collect();
        class_names.sort_unstable();
        for name in class_names {
            visit(name, &dependencies, &mut visited, &mut sorted_names);
        }

        sorted_names
            .into_iter()
            .filter_map(|name| class_map.get(name).copied())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagram_extend_concatenates_all_fields() {
        let mut a = Diagram::new();
        a.add_class(ClassNode {
            name: "A1".to_string(),
            type_params: None,
            class_type: ClassType::Regular,
            attributes: vec![],
            methods: vec![],
        });
        a.add_relationship(RelationshipEdge {
            from: "A1".to_string(),
            to: "Base".to_string(),
            relation_type: RelationType::Inheritance,
        });
        a.add_composition(CompositionEdge {
            container: "A1".to_string(),
            contained: "Widget".to_string(),
        });

        let mut b = Diagram::new();
        b.add_class(ClassNode {
            name: "B1".to_string(),
            type_params: None,
            class_type: ClassType::Abstract,
            attributes: vec![],
            methods: vec![],
        });
        b.add_relationship(RelationshipEdge {
            from: "B1".to_string(),
            to: "Base".to_string(),
            relation_type: RelationType::Implementation,
        });
        b.add_composition(CompositionEdge {
            container: "B1".to_string(),
            contained: "Gadget".to_string(),
        });

        a.extend(b);

        assert_eq!(a.classes.len(), 2);
        assert_eq!(a.classes[0].name, "A1");
        assert_eq!(a.classes[1].name, "B1");

        assert_eq!(a.relationships.len(), 2);
        assert_eq!(a.relationships[0].to, "Base");
        assert_eq!(a.relationships[1].to, "Base");

        assert_eq!(a.compositions.len(), 2);
        assert_eq!(a.compositions[0].contained, "Widget");
        assert_eq!(a.compositions[1].contained, "Gadget");

        assert!(a.is_abstract_or_interface("B1"));
        assert!(!a.is_abstract_or_interface("A1"));
    }
}
