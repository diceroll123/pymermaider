use crate::mermaid_escape::MermaidEscape;
use crate::renderer::*;
use itertools::Itertools;

const TAB: &str = "    ";

/// Mermaid-specific renderer implementation
pub struct MermaidRenderer {}

impl MermaidRenderer {
    pub fn new() -> Self {
        Self {}
    }

    fn indent(&self, indent_level: usize) -> String {
        TAB.repeat(indent_level)
    }

    fn format_visibility(&self, visibility: Visibility) -> char {
        match visibility {
            Visibility::Public => '+',
            Visibility::Private => '-',
            Visibility::Protected => '#',
        }
    }

    fn format_class_type(&self, class_type: ClassType) -> Option<&'static str> {
        match class_type {
            ClassType::Regular => None,
            ClassType::Abstract => Some("<<abstract>>"),
            ClassType::Interface => Some("<<interface>>"),
            ClassType::Enumeration => Some("<<enumeration>>"),
            ClassType::Dataclass => Some("<<dataclass>>"),
            ClassType::Final => Some("<<final>>"),
        }
    }

    fn has_class_body(&self, class: &ClassNode) -> bool {
        !class.attributes.is_empty()
            || !class.methods.is_empty()
            || class.class_type != ClassType::Regular
    }

    fn render_class_annotation(
        &self,
        output: &mut String,
        inner_indent: &str,
        class_type: ClassType,
    ) {
        if let Some(annotation) = self.format_class_type(class_type) {
            output.push_str(inner_indent);
            output.push_str(annotation);
            output.push('\n');
        }
    }

    fn render_attribute(&self, output: &mut String, inner_indent: &str, attr: &Attribute) {
        output.push_str(inner_indent);
        output.push(self.format_visibility(attr.visibility));
        output.push(' ');
        output.push_str(&attr.type_annotation);
        output.push(' ');
        output.push_str(&attr.name.escape_underscores());
        output.push('\n');
    }

    fn render_method(&self, output: &mut String, inner_indent: &str, method: &MethodSignature) {
        output.push_str(inner_indent);
        output.push(self.format_visibility(method.visibility));
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

    fn render_relationship_symbol(&self, relation_type: RelationType) -> &'static str {
        match relation_type {
            RelationType::Inheritance => "--|>",
            RelationType::Implementation => "..|>",
        }
    }
}

impl Default for MermaidRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramRenderer for MermaidRenderer {
    fn render_header(&self, title: Option<&str>) -> String {
        let mut output = String::new();

        if let Some(title) = title {
            output.push_str("---\n");
            output.push_str(&format!("title: {}\n", title));
            output.push_str("---\n");
        }

        output.push_str("classDiagram\n");
        output
    }

    fn render_class(&self, class: &ClassNode) -> String {
        let mut output = String::new();
        let indent = self.indent(1);
        let inner_indent = self.indent(2);

        // Class declaration
        output.push_str(&indent);
        output.push_str("class ");
        output.push_str(&class.name);

        // Type parameters (generics)
        if let Some(ref type_params) = class.type_params {
            output.push_str(&format!(" ~{}~", type_params));
        }

        if self.has_class_body(class) {
            output.push_str(" {\n");

            // Class type annotation
            self.render_class_annotation(&mut output, &inner_indent, class.class_type);

            // Attributes
            for attr in &class.attributes {
                self.render_attribute(&mut output, &inner_indent, attr);
            }

            // Methods
            for method in &class.methods {
                self.render_method(&mut output, &inner_indent, method);
            }

            output.push_str(&indent);
            output.push('}');
        }

        output.push_str("\n\n");
        output
    }

    fn render_relationship(&self, relationship: &RelationshipEdge) -> String {
        let symbol = self.render_relationship_symbol(relationship.relation_type);

        format!(
            "{}{} {} {}\n",
            self.indent(1),
            relationship.from,
            symbol,
            relationship.to
        )
    }

    fn render_composition(&self, composition: &CompositionEdge) -> String {
        format!(
            "{}{} *-- {}\n",
            self.indent(1),
            composition.container,
            composition.contained
        )
    }

    fn render_diagram(&self, diagram: &Diagram) -> Option<String> {
        if diagram.is_empty() {
            return None;
        }

        let mut output = String::with_capacity(1024);

        // Header
        output.push_str(&self.render_header(diagram.title.as_deref()));

        // Classes (deduplicated)
        for class in diagram.classes.iter().unique_by(|c| &c.name) {
            output.push_str(&self.render_class(class));
        }

        // Relationships (deduplicated)
        let unique_relationships: Vec<_> = diagram.relationships.iter().unique().collect();
        if !unique_relationships.is_empty() {
            let relationship_strs: Vec<String> = unique_relationships
                .iter()
                .map(|rel| self.render_relationship(rel))
                .collect();
            output.push_str(&relationship_strs.join("\n"));
        }

        // Compositions (deduplicated)
        let unique_compositions: Vec<_> = diagram.compositions.iter().unique().collect();
        if !unique_compositions.is_empty() {
            if !unique_relationships.is_empty() {
                output.push('\n');
            }
            let composition_strs: Vec<String> = unique_compositions
                .iter()
                .map(|comp| self.render_composition(comp))
                .collect();
            output.push_str(&composition_strs.join("\n"));
        }

        output = output.trim_end().to_owned();
        output.push('\n');

        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_class() {
        let renderer = MermaidRenderer::new();

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

        let output = renderer.render_class(&class);
        assert!(output.contains("class Person"));
        assert!(output.contains("+ str name"));
        assert!(output.contains("+ greet(self) str"));
    }

    #[test]
    fn test_render_relationship() {
        let renderer = MermaidRenderer::new();

        let rel = RelationshipEdge {
            from: "Dog".to_string(),
            to: "Animal".to_string(),
            relation_type: RelationType::Inheritance,
        };

        let output = renderer.render_relationship(&rel);
        assert!(output.contains("Dog --|> Animal"));
    }
}
