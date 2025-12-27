use crate::ast;
use crate::checker::Checker;
use crate::class_helpers::{ClassDefHelpers, QualifiedNameHelpers};
use crate::class_type_detector::ClassTypeDetector;
use crate::parameter_generator::ParameterGenerator;
use crate::renderer::*;
use crate::type_analyzer;
use indexmap::IndexSet;
use ruff_linter::source_kind::SourceKind;
use ruff_linter::Locator;
use ruff_python_ast::name::QualifiedName;
use ruff_python_ast::{Expr, Number, PySourceType};
use ruff_python_codegen::Stylist;
use ruff_python_parser::parse_unchecked_source;
use ruff_python_semantic::analyze::visibility::{
    is_abstract, is_classmethod, is_final, is_overload, is_override, is_staticmethod,
};
use ruff_python_semantic::{Module, ModuleKind, ModuleSource, SemanticModel};
use ruff_python_stdlib::typing::simple_magic_return_type;
use std::path::Path;

/// Represents a class member (attribute or method) during processing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ClassMember {
    Attribute(Attribute),
    Method(MethodSignature),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BaseKind {
    Skip,
    InheritanceTarget {
        name: String,
        is_abstract_or_protocol: bool,
    },
}

pub struct ClassDiagram {
    diagram: Diagram,
    pub path: String,
}

impl Default for ClassDiagram {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassDiagram {
    pub fn new() -> Self {
        Self {
            diagram: Diagram::new(None),
            path: String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diagram.is_empty()
    }

    pub fn render(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let title = if self.path.is_empty() {
            None
        } else {
            Some(self.path.as_str())
        };

        crate::mermaid_renderer::render_diagram(&self.diagram, title)
    }

    pub fn add_class(
        &mut self,
        checker: &Checker,
        class: &ast::StmtClassDef,
        _indent_level: usize,
    ) {
        let class_name = class.name.to_string();

        // Find generic type parameters - either from explicit [T] syntax or Generic[T] bases
        let mut generic_type_var = None;

        if let Some(params) = &class.type_params {
            // Explicit type parameters via [T] syntax (Python 3.12+)
            // Extract the raw type params from the source
            let raw_params = checker.locator().slice(params.as_ref());
            // Remove the brackets to get just the type names
            generic_type_var = Some(
                raw_params
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
            );
        } else {
            // Check for Generic[T] in bases
            for base in class.bases() {
                if let Some(type_var) = type_analyzer::extract_generic_params(base, checker) {
                    generic_type_var = Some(type_var);
                    break;
                }
            }
        }

        // Detect composition relationships from class attributes
        let mut composition_types: IndexSet<String> = IndexSet::new();
        for stmt in &class.body {
            if let ast::Stmt::AnnAssign(ast::StmtAnnAssign { annotation, .. }) = stmt {
                composition_types.extend(type_analyzer::extract_composition_types(
                    annotation.as_ref(),
                    checker,
                ));
            }
        }

        // Process class body statements
        let mut members: IndexSet<ClassMember> = IndexSet::new();
        for stmt in &class.body {
            if let Some(member) = self.process_stmt_to_member(checker, stmt) {
                members.insert(member);
            }
        }

        // Detect class type using ClassTypeDetector
        let detector = ClassTypeDetector::new(checker);
        let class_type = detector.detect_type(class);
        let class_is_enum = class.is_enum(checker.semantic());

        // Split members into attributes and methods
        let mut attributes = Vec::new();
        let mut methods = Vec::new();
        for member in members {
            match member {
                ClassMember::Attribute(attr) => attributes.push(attr),
                ClassMember::Method(method) => methods.push(method),
            }
        }

        let class_node = ClassNode {
            name: class_name.clone(),
            type_params: generic_type_var,
            class_type,
            attributes,
            methods,
        };

        self.diagram.add_class(class_node);

        // Handle inheritance relationships
        for base in class.bases() {
            match self.classify_base(checker, &detector, base, class_is_enum) {
                BaseKind::Skip => continue,
                BaseKind::InheritanceTarget {
                    name,
                    is_abstract_or_protocol,
                } => {
                    let rel = RelationshipEdge {
                        from: class_name.clone(),
                        to: name,
                        relation_type: if is_abstract_or_protocol {
                            RelationType::Implementation
                        } else {
                            RelationType::Inheritance
                        },
                    };
                    self.diagram.add_relationship(rel);
                }
            }
        }

        // Add composition relationships
        for comp_type in &composition_types {
            // Extract just the class name (remove module prefix if present)
            let comp_display = comp_type.split('.').next_back().unwrap_or(comp_type);

            let comp = CompositionEdge {
                container: class_name.clone(),
                contained: comp_display.to_string(),
            };
            self.diagram.add_composition(comp);
        }
    }

    /// Process a statement into an Attribute or MethodSignature
    fn process_stmt_to_member(&self, checker: &Checker, stmt: &ast::Stmt) -> Option<ClassMember> {
        match stmt {
            ast::Stmt::AnnAssign(ast::StmtAnnAssign {
                target,
                annotation,
                simple,
                ..
            }) => {
                if !simple {
                    return None;
                }

                let Expr::Name(ast::ExprName { id: target, .. }) = target.as_ref() else {
                    return None;
                };

                let target_name = target.to_string();
                let annotation_name = checker.generator().expr(annotation.as_ref());
                let is_private = target_name.starts_with('_');

                Some(ClassMember::Attribute(Attribute {
                    name: target_name,
                    type_annotation: annotation_name,
                    visibility: if is_private {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    },
                }))
            }

            ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) => {
                // Handle simple assignments (like enum members)
                let value_type = match value.as_ref() {
                    Expr::BoolOp(_) => "bool",
                    Expr::BinOp(_) | Expr::UnaryOp(_) => "int",
                    Expr::Lambda(_) => "Callable",
                    Expr::DictComp(_) | Expr::Dict(_) => "dict",
                    Expr::Set(_) | Expr::SetComp(_) => "set",
                    Expr::FString(_) | Expr::StringLiteral(_) => "str",
                    Expr::NoneLiteral(_) => "None",
                    Expr::BooleanLiteral(_) => "bool",
                    Expr::BytesLiteral(_) => "bytes",
                    Expr::EllipsisLiteral(_) => "...",
                    Expr::ListComp(_) | Expr::List(_) => "list",
                    Expr::Tuple(_) => "tuple",
                    Expr::NumberLiteral(inner) => match inner.value {
                        Number::Int(_) => "int",
                        Number::Float(_) => "float",
                        Number::Complex { .. } => "complex",
                    },
                    _ => "",
                };

                // For now, just handle the first target (typical for enums and simple assignments)
                if let Some(Expr::Name(ast::ExprName { id: target, .. })) = targets.first() {
                    let target_name = target.to_string();

                    return Some(ClassMember::Attribute(Attribute {
                        name: target_name,
                        type_annotation: if value_type.is_empty() {
                            "Any".to_string()
                        } else {
                            value_type.to_string()
                        },
                        visibility: Visibility::Public, // Simple assignments are always public
                    }));
                }

                None
            }

            ast::Stmt::FunctionDef(ast::StmtFunctionDef {
                name,
                is_async,
                parameters,
                returns,
                decorator_list,
                ..
            }) => {
                let is_private = name.starts_with('_');
                let is_static = is_staticmethod(decorator_list, checker.semantic());

                let mut param_gen = ParameterGenerator::new();
                param_gen.unparse_parameters(parameters);
                let params = param_gen.generate();

                let returns = match returns {
                    Some(target) => Some(checker.generator().expr(target.as_ref())),
                    None => simple_magic_return_type(name).map(String::from),
                };

                let mut decorators = vec![];
                if is_final(decorator_list, checker.semantic()) {
                    decorators.push("@final".to_string());
                }
                if is_classmethod(decorator_list, checker.semantic()) {
                    decorators.push("@classmethod".to_string());
                } else if is_static {
                    decorators.push("@staticmethod".to_string());
                }
                if is_overload(decorator_list, checker.semantic()) {
                    decorators.push("@overload".to_string());
                }
                if is_override(decorator_list, checker.semantic()) {
                    decorators.push("@override".to_string());
                }

                Some(ClassMember::Method(MethodSignature {
                    name: name.to_string(),
                    parameters: params,
                    return_type: returns,
                    visibility: if is_private {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    },
                    is_static,
                    is_abstract: is_abstract(decorator_list, checker.semantic()),
                    is_async: *is_async,
                    decorators,
                }))
            }

            _ => None,
        }
    }

    fn classify_base(
        &self,
        checker: &Checker,
        detector: &ClassTypeDetector,
        base: &ast::Expr,
        class_is_enum: bool,
    ) -> BaseKind {
        // Enums are a special case: we don't draw inheritance relationships for enum bases.
        if class_is_enum {
            return BaseKind::Skip;
        }

        // Skip generic parameter carrier bases like Generic[T].
        if type_analyzer::extract_generic_params(base, checker).is_some() {
            return BaseKind::Skip;
        }

        if checker
            .semantic()
            .resolve_qualified_name(base)
            .is_some_and(|name| {
                matches!(name.segments(), ["typing", "Generic"])
                    || matches!(name.segments(), ["" | "builtins", "object"])
                    || matches!(name.segments(), ["abc", "ABC" | "ABCMeta"])
                    || matches!(
                        name.segments(),
                        ["typing" | "typing_extensions", "Protocol"]
                    )
            })
        {
            return BaseKind::Skip;
        }

        let base_name = match checker.semantic().resolve_qualified_name(base) {
            Some(base_name) => base_name.normalize_name(),
            None => {
                let name = checker.locator().slice(base);
                QualifiedName::user_defined(name).normalize_name()
            }
        };

        // Extract just the base class name without the generic specialization.
        let base_display = base_name
            .split('[')
            .next()
            .unwrap_or(&base_name)
            .trim_matches('`')
            .to_string();

        // Check if the base class is abstract or a protocol (either built-in or user-defined).
        let base_is_abstract_or_protocol = self.diagram.is_abstract_or_interface(&base_display)
            || detector.is_stdlib_abstract_or_protocol(base);

        BaseKind::InheritanceTarget {
            name: base_display,
            is_abstract_or_protocol: base_is_abstract_or_protocol,
        }
    }

    /// Add source code to the diagram (for stdin/WASM - uses Python defaults)
    pub fn add_source(&mut self, source: String) {
        self.add_source_with_options(source, PySourceType::Python, ModuleKind::Module);
    }

    /// Add source code from a file path (infers source type and module kind)
    pub fn add_file(&mut self, source: String, path: &Path) {
        let source_type = PySourceType::from(path);
        let module_kind = Self::module_kind_for_path(path);
        self.add_source_with_options(source, source_type, module_kind);
    }

    fn add_source_with_options(
        &mut self,
        source: String,
        source_type: PySourceType,
        module_kind: ModuleKind,
    ) {
        let source_kind = SourceKind::Python(source);

        let parsed = Self::parse_python(source_kind.source_code(), source_type);
        let mut checker = Self::build_checker(
            &parsed.stylist,
            &parsed.locator,
            &parsed.python_ast,
            module_kind,
        );
        checker.see_imports(&parsed.python_ast);

        self.add_classes_from_ast(&checker, &parsed.python_ast);
    }

    fn add_classes_from_ast(&mut self, checker: &Checker, python_ast: &[ast::Stmt]) {
        for stmt in python_ast {
            if let ast::Stmt::ClassDef(class) = stmt {
                // we only care about class definitions
                self.add_class(checker, class, 1);
            }
        }
    }

    fn module_kind_for_path(path: &Path) -> ModuleKind {
        if path.ends_with("__init__.py") {
            ModuleKind::Package
        } else {
            ModuleKind::Module
        }
    }

    fn parse_python(source: &str, source_type: PySourceType) -> ParsedPython<'_> {
        let parsed = parse_unchecked_source(source, source_type);
        let stylist = Stylist::from_tokens(parsed.tokens(), source);
        let python_ast = parsed.into_suite();

        ParsedPython {
            python_ast,
            locator: Locator::new(source),
            stylist,
        }
    }

    fn build_checker<'a>(
        stylist: &'a Stylist<'a>,
        locator: &'a Locator<'a>,
        python_ast: &'a [ast::Stmt],
        module_kind: ModuleKind,
    ) -> Checker<'a> {
        // Use a static dummy path for the semantic model (it's only used for diagnostics)
        static DUMMY_PATH: &str = "";
        let dummy = Path::new(DUMMY_PATH);

        let module = Module {
            kind: module_kind,
            source: ModuleSource::File(dummy),
            python_ast,
            name: None,
        };
        let semantic = SemanticModel::new(&[], dummy, module);
        Checker::new(stylist, locator, semantic)
    }
}

struct ParsedPython<'a> {
    python_ast: Vec<ast::Stmt>,
    locator: Locator<'a>,
    stylist: Stylist<'a>,
}

#[cfg(test)]
mod tests;
