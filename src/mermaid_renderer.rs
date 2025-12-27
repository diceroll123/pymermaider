use crate::mermaid_escape::MermaidEscape;
use crate::renderer::{
    Attribute, ClassNode, ClassType, CompositionEdge, Diagram, DiagramDirection, MethodSignature,
    RelationType, RelationshipEdge, Visibility,
};
use indexmap::IndexSet;

const TAB: &str = "    ";

fn indent(indent_level: usize) -> String {
    TAB.repeat(indent_level)
}

fn format_visibility(visibility: Visibility) -> char {
    match visibility {
        Visibility::Public => '+',
        Visibility::Private => '-',
        Visibility::Protected => '#',
    }
}

fn format_class_type(class_type: ClassType) -> Option<&'static str> {
    match class_type {
        ClassType::Regular => None,
        ClassType::Abstract => Some("<<abstract>>"),
        ClassType::Interface => Some("<<interface>>"),
        ClassType::Enumeration => Some("<<enumeration>>"),
        ClassType::Dataclass => Some("<<dataclass>>"),
        ClassType::Final => Some("<<final>>"),
    }
}

fn has_class_body(class: &ClassNode) -> bool {
    !class.attributes.is_empty()
        || !class.methods.is_empty()
        || class.class_type != ClassType::Regular
}

fn render_class_annotation(output: &mut String, inner_indent: &str, class_type: ClassType) {
    if let Some(annotation) = format_class_type(class_type) {
        output.push_str(inner_indent);
        output.push_str(annotation);
        output.push('\n');
    }
}

fn render_attribute(output: &mut String, inner_indent: &str, attr: &Attribute) {
    output.push_str(inner_indent);
    output.push(format_visibility(attr.visibility));
    output.push(' ');
    output.push_str(&attr.type_annotation);
    output.push(' ');
    output.push_str(&attr.name.escape_underscores());
    output.push('\n');
}

fn render_method(output: &mut String, inner_indent: &str, method: &MethodSignature) {
    output.push_str(inner_indent);
    output.push(format_visibility(method.visibility));
    output.push(' ');

    // Decorators
    for decorator in &method.decorators {
        output.push_str(decorator);
        output.push(' ');
    }

    // Async modifier
    if method.is_async {
        output.push_str("async ");
    }

    // Method signature
    output.push_str(&method.name.escape_underscores());
    output.push('(');
    output.push_str(&method.parameters);
    output.push(')');

    // Return type
    if let Some(ref return_type) = method.return_type {
        output.push(' ');
        output.push_str(return_type);
    }

    // Classifiers
    if method.is_abstract {
        output.push('*');
    } else if method.is_static {
        output.push('$');
    }

    output.push('\n');
}

fn render_relationship_symbol(relation_type: RelationType) -> &'static str {
    match relation_type {
        RelationType::Inheritance => "--|>",
        RelationType::Implementation => "..|>",
    }
}

pub fn render_header(title: Option<&str>, direction: DiagramDirection) -> String {
    let mut output = String::new();

    if let Some(title) = title {
        output.push_str("---\n");
        output.push_str(&format!("title: {}\n", title));
        output.push_str("---\n");
    }

    output.push_str("classDiagram\n");

    // Only emit direction if non-default
    if direction != DiagramDirection::default() {
        output.push_str(&format!("{}direction {}\n\n", indent(1), direction));
    }

    output
}

pub fn render_class(class: &ClassNode) -> String {
    let mut output = String::new();
    let outer_indent = indent(1);
    let inner_indent = indent(2);

    // Class declaration
    output.push_str(&outer_indent);
    output.push_str("class ");
    output.push_str(&class.name);

    // Type parameters (generics)
    if let Some(ref type_params) = class.type_params {
        output.push_str(&format!(" ~{}~", type_params));
    }

    if has_class_body(class) {
        output.push_str(" {\n");

        // Class type annotation
        render_class_annotation(&mut output, &inner_indent, class.class_type);

        // Attributes
        for attr in &class.attributes {
            render_attribute(&mut output, &inner_indent, attr);
        }

        // Methods
        for method in &class.methods {
            render_method(&mut output, &inner_indent, method);
        }

        output.push_str(&outer_indent);
        output.push('}');
    }

    output.push_str("\n\n");
    output
}

pub fn render_relationship(relationship: &RelationshipEdge) -> String {
    let symbol = render_relationship_symbol(relationship.relation_type);

    format!(
        "{}{} {} {}\n",
        indent(1),
        relationship.from,
        symbol,
        relationship.to
    )
}

pub fn render_composition(composition: &CompositionEdge) -> String {
    format!(
        "{}{} *-- {}\n",
        indent(1),
        composition.container,
        composition.contained
    )
}
/// Render a full Mermaid class diagram.
pub fn render_diagram(
    diagram: &Diagram,
    title: Option<&str>,
    direction: DiagramDirection,
) -> Option<String> {
    if diagram.is_empty() {
        return None;
    }

    let mut output = String::with_capacity(1024);
    output.push_str(&render_header(title, direction));

    for class in diagram.classes_topologically_sorted_unique() {
        output.push_str(&render_class(class));
    }

    // Relationships (deduped; stable order)
    let unique_relationships: IndexSet<_> = diagram.relationships.iter().collect();
    if !unique_relationships.is_empty() {
        for (idx, rel) in unique_relationships.iter().enumerate() {
            output.push_str(&render_relationship(rel));
            if idx + 1 < unique_relationships.len() {
                output.push('\n');
            }
        }
    }

    // Compositions (deduped; stable order)
    let unique_compositions: IndexSet<_> = diagram.compositions.iter().collect();
    if !unique_compositions.is_empty() {
        if !unique_relationships.is_empty() {
            output.push('\n');
        }

        for (idx, comp) in unique_compositions.iter().enumerate() {
            output.push_str(&render_composition(comp));
            if idx + 1 < unique_compositions.len() {
                output.push('\n');
            }
        }
    }

    output = output.trim_end().to_owned();
    output.push('\n');
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_class() {
        let class = ClassNode {
            name: "Person".to_string(),
            type_params: None,
            class_type: ClassType::Regular,
            attributes: vec![Attribute {
                name: "name".to_string(),
                type_annotation: "str".to_string(),
                visibility: Visibility::Public,
            }],
            methods: vec![MethodSignature {
                name: "greet".to_string(),
                parameters: "self".to_string(),
                return_type: Some("str".to_string()),
                visibility: Visibility::Public,
                is_static: false,
                is_abstract: false,
                is_async: false,
                decorators: vec![],
            }],
        };

        let output = render_class(&class);
        assert!(output.contains("class Person"));
        assert!(output.contains("+ str name"));
        assert!(output.contains("+ greet(self) str"));
    }

    #[test]
    fn test_render_relationship() {
        let rel = RelationshipEdge {
            from: "Dog".to_string(),
            to: "Animal".to_string(),
            relation_type: RelationType::Inheritance,
        };

        let output = render_relationship(&rel);
        assert!(output.contains("Dog --|> Animal"));
    }
}
