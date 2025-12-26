/// Trait for rendering class diagrams in different formats
pub trait DiagramRenderer {
    /// Render the diagram header/preamble
    fn render_header(&self, title: Option<&str>) -> String;

    /// Render a single class definition
    fn render_class(&self, class: &ClassNode) -> String;

    /// Render an inheritance/implementation relationship
    fn render_relationship(&self, relationship: &RelationshipEdge) -> String;

    /// Render a composition relationship
    fn render_composition(&self, composition: &CompositionEdge) -> String;

    /// Render the complete diagram, returns None if diagram is empty
    fn render_diagram(&self, diagram: &Diagram) -> Option<String> {
        // Check if diagram is empty
        if diagram.is_empty() {
            return None;
        }

        let mut output = String::with_capacity(1024);

        // Header
        output.push_str(&self.render_header(diagram.title.as_deref()));

        // Classes
        for class in &diagram.classes {
            output.push_str(&self.render_class(class));
        }

        // Relationships
        if !diagram.relationships.is_empty() {
            for relationship in &diagram.relationships {
                output.push_str(&self.render_relationship(relationship));
            }
        }

        // Compositions
        if !diagram.compositions.is_empty() {
            for composition in &diagram.compositions {
                output.push_str(&self.render_composition(composition));
            }
        }

        Some(output)
    }
}

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

/// The complete diagram structure
#[derive(Debug, Clone)]
pub struct Diagram {
    pub title: Option<String>,
    pub classes: Vec<ClassNode>,
    pub relationships: Vec<RelationshipEdge>,
    pub compositions: Vec<CompositionEdge>,
}

impl Diagram {
    pub fn new(title: Option<String>) -> Self {
        Self {
            title,
            classes: Vec::new(),
            relationships: Vec::new(),
            compositions: Vec::new(),
        }
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

    /// Sort classes topologically based on their relationships and compositions.
    /// This ensures that base classes and composed classes appear before derived/container classes.
    pub fn sort_classes_topologically(&mut self) {
        use std::collections::{HashMap, HashSet};

        let mut class_map: HashMap<String, ClassNode> = HashMap::new();

        // Build a map of class names to their ClassNodes
        for class in &self.classes {
            class_map.insert(class.name.clone(), class.clone());
        }

        // Build dependency graph from relationships and compositions
        let mut dependencies: HashMap<String, HashSet<String>> = HashMap::new();

        for relationship in &self.relationships {
            dependencies
                .entry(relationship.from.clone())
                .or_default()
                .insert(relationship.to.clone());
        }

        for composition in &self.compositions {
            dependencies
                .entry(composition.container.clone())
                .or_default()
                .insert(composition.contained.clone());
        }

        // Topological sort with grouping
        let mut visited = HashSet::new();
        let mut sorted = Vec::new();

        fn visit(
            name: &str,
            dependencies: &HashMap<String, HashSet<String>>,
            visited: &mut HashSet<String>,
            sorted: &mut Vec<String>,
        ) {
            if visited.contains(name) {
                return;
            }
            visited.insert(name.to_string());

            // Visit dependencies first in sorted order for consistency
            if let Some(deps) = dependencies.get(name) {
                let mut sorted_deps: Vec<_> = deps.iter().cloned().collect();
                sorted_deps.sort();
                for dep in sorted_deps {
                    visit(&dep, dependencies, visited, sorted);
                }
            }

            sorted.push(name.to_string());
        }

        // Visit all classes in sorted order for consistent output
        let mut class_names: Vec<_> = class_map.keys().cloned().collect();
        class_names.sort();
        for name in class_names {
            visit(&name, &dependencies, &mut visited, &mut sorted);
        }

        // Replace classes with sorted version
        self.classes = sorted
            .iter()
            .filter_map(|name| class_map.get(name).cloned())
            .collect();
    }
}
