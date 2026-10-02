use crate::analysis::checker::Checker;
use crate::analysis::class_helpers::{
    is_abc_qualified_name, ClassDefHelpers, QualifiedNameHelpers,
};
use crate::analysis::class_type_detector::ClassTypeDetector;
use crate::analysis::parameter_generator::ParameterGenerator;
use crate::analysis::type_analyzer;
use crate::ast;
use crate::render::renderer::{
    Attribute, ClassNode, CompositionEdge, CompositionKind, Diagram, MethodSignature, RelationType,
    RelationshipEdge, Visibility,
};
use indexmap::IndexSet;
use ruff_linter::source_kind::SourceKind;
use ruff_linter::Locator;
use ruff_python_ast::name::{QualifiedName, UnqualifiedName};
use ruff_python_ast::{Expr, Number, PySourceType};
use ruff_python_codegen::Stylist;
use ruff_python_parser::parse_unchecked_source;
use ruff_python_semantic::analyze::visibility::{
    is_abstract, is_classmethod, is_final, is_overload, is_override, is_property, is_staticmethod,
};
use ruff_python_semantic::{Module, ModuleKind, ModuleSource, SemanticModel};
use ruff_python_stdlib::typing::simple_magic_return_type;
use std::collections::HashMap;
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
        is_stdlib_abstract_or_protocol: bool,
    },
}

pub struct ClassDiagram {
    diagram: Diagram,
    options: crate::render::mermaid_renderer::RenderOptions,
    pub path: String,
    show_title: bool,
    /// Syntax errors found in the added sources. Parsing is lenient, so output
    /// may still be produced for the valid parts.
    syntax_errors: Vec<String>,
    /// Dotted module path prepended to every emitted class name (empty for none).
    module_prefix: String,
}

impl Default for ClassDiagram {
    fn default() -> Self {
        Self::new(crate::render::mermaid_renderer::RenderOptions::default())
    }
}

impl ClassDiagram {
    #[must_use]
    pub fn new(options: crate::render::mermaid_renderer::RenderOptions) -> Self {
        Self {
            diagram: Diagram::new(),
            options,
            path: String::new(),
            show_title: true,
            syntax_errors: Vec::new(),
            module_prefix: String::new(),
        }
    }

    /// Whether `path` is rendered as the diagram title (it still names output files).
    pub const fn set_show_title(&mut self, show: bool) {
        self.show_title = show;
    }

    /// Syntax errors found while parsing added sources, as `line N: message`.
    #[must_use]
    pub fn syntax_errors(&self) -> &[String] {
        &self.syntax_errors
    }

    /// Qualify every class emitted from now on with a dotted module path,
    /// e.g. `models.user` turns `User` into `models.user.User`.
    pub fn set_module_prefix(&mut self, prefix: &str) {
        prefix.clone_into(&mut self.module_prefix);
    }

    pub const fn set_hide_private_members(&mut self, hide: bool) {
        self.options.hide_private_members = hide;
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.diagram.is_empty()
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.syntax_errors.extend(other.syntax_errors);
        self.diagram.extend(other.diagram);
        self.diagram.resolve_references();
        self.diagram.finalize_relation_types();
        self
    }

    #[must_use]
    pub fn render(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let title = if self.path.is_empty() || !self.show_title {
            None
        } else {
            Some(self.path.as_str())
        };

        crate::render::mermaid_renderer::render_diagram(&self.diagram, title, &self.options)
    }

    pub fn add_class(
        &mut self,
        checker: &Checker,
        class: &ast::StmtClassDef,
        _indent_level: usize,
    ) {
        self.add_class_in_scope(checker, class, &[]);
    }

    /// Add a class nested inside `enclosing` (outermost first). Nested classes are
    /// emitted under their dotted path, e.g. `Outer.Inner`.
    fn add_class_in_scope(
        &mut self,
        checker: &Checker,
        class: &ast::StmtClassDef,
        enclosing: &[&str],
    ) {
        let local_name = class.name.as_str();
        let class_name = if enclosing.is_empty() && self.module_prefix.is_empty() {
            local_name.to_owned()
        } else {
            let path = std::iter::once(self.module_prefix.as_str())
                .filter(|prefix| !prefix.is_empty())
                .chain(enclosing.iter().copied())
                .chain(std::iter::once(local_name))
                .collect::<Vec<_>>()
                .join(".");
            let normalized = QualifiedName::user_defined(&path).normalize_name();
            normalized
        };
        self.diagram
            .register_name(local_name, &class_name, enclosing.len());

        // Find generic type parameters - either from explicit [T] syntax or Generic[T] bases
        let generic_type_var = class.type_params.as_ref().map_or_else(
            || {
                // Check for Generic[T] in bases
                let mut found = None;
                for base in class.bases() {
                    if let Some(type_var) = type_analyzer::extract_generic_params(base, checker) {
                        found = Some(type_var);
                        break;
                    }
                }
                found
            },
            |params| {
                // Explicit type parameters via [T] syntax (Python 3.12+)
                let raw_params = checker.locator().slice(params.as_ref());
                // Remove the brackets to get just the type names
                Some(
                    raw_params
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .to_owned(),
                )
            },
        );

        // Detect composition relationships from class attributes and collect members
        let mut composition_types: IndexSet<(String, bool)> = IndexSet::new();
        let mut members: IndexSet<ClassMember> = IndexSet::new();
        for stmt in Self::flatten_class_body(&class.body) {
            if let ast::Stmt::AnnAssign(ast::StmtAnnAssign { annotation, .. }) = stmt {
                composition_types.extend(type_analyzer::extract_composition_types(
                    annotation.as_ref(),
                    checker,
                ));
            }
            if let Some(member) = Self::process_stmt_to_member(checker, stmt) {
                members.insert(member);
            }
        }

        // Collect instance attributes assigned through `self` in `__init__`
        for (attr, annotation) in Self::collect_instance_attributes(checker, class) {
            if let Some(annotation) = annotation {
                composition_types.extend(type_analyzer::extract_composition_types(
                    annotation, checker,
                ));
            }
            let already_declared = members
                .iter()
                .any(|m| matches!(m, ClassMember::Attribute(a) if a.name == attr.name));
            if !already_declared {
                members.insert(ClassMember::Attribute(attr));
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
            let BaseKind::InheritanceTarget {
                name,
                is_stdlib_abstract_or_protocol,
            } = self.classify_base(checker, &detector, base, class_is_enum)
            else {
                continue;
            };
            let rel = RelationshipEdge {
                from: class_name.clone(),
                to: name,
                // Final type is decided in `finalize_relation_types` once every class is known
                relation_type: RelationType::Inheritance,
                is_stdlib_abstract_or_protocol,
            };
            self.diagram.add_relationship(rel);
        }

        // Add composition relationships
        // A type that is also held directly keeps only the stronger composition edge.
        for (comp_type, is_aggregation) in &composition_types {
            if *is_aggregation && composition_types.contains(&(comp_type.clone(), false)) {
                continue;
            }
            // Imported types keep their module path (backticked, e.g. `pathlib.Path`) so
            // same-named types from different modules stay distinct. Bare names are
            // local classes and are resolved to their emitted name later.
            let comp = CompositionEdge {
                container: class_name.clone(),
                contained: QualifiedName::user_defined(comp_type).normalize_name(),
                kind: if *is_aggregation {
                    CompositionKind::Aggregation
                } else {
                    CompositionKind::Composition
                },
            };
            self.diagram.add_composition(comp);
        }
    }

    /// Class body statements, including those nested in `if`/`try`/`with` blocks
    /// (e.g. members defined under `if TYPE_CHECKING:` or `try: ... except ImportError:`).
    fn flatten_class_body(body: &[ast::Stmt]) -> Vec<&ast::Stmt> {
        let mut out = Vec::new();
        for stmt in body {
            match stmt {
                ast::Stmt::If(ast::StmtIf {
                    body,
                    elif_else_clauses,
                    ..
                }) => {
                    out.extend(Self::flatten_class_body(body));
                    for clause in elif_else_clauses {
                        out.extend(Self::flatten_class_body(&clause.body));
                    }
                }
                ast::Stmt::Try(ast::StmtTry {
                    body,
                    handlers,
                    orelse,
                    finalbody,
                    ..
                }) => {
                    out.extend(Self::flatten_class_body(body));
                    for handler in handlers {
                        let ast::ExceptHandler::ExceptHandler(h) = handler;
                        out.extend(Self::flatten_class_body(&h.body));
                    }
                    out.extend(Self::flatten_class_body(orelse));
                    out.extend(Self::flatten_class_body(finalbody));
                }
                ast::Stmt::With(ast::StmtWith { body, .. }) => {
                    out.extend(Self::flatten_class_body(body));
                }
                other => out.push(other),
            }
        }
        out
    }

    /// Find `self.x = ...` / `self.x: T = ...` assignments in `__init__`.
    /// Returns each attribute along with its explicit annotation expression, if any.
    fn collect_instance_attributes<'a>(
        checker: &Checker,
        class: &'a ast::StmtClassDef,
    ) -> Vec<(Attribute, Option<&'a Expr>)> {
        fn walk<'a>(
            checker: &Checker,
            stmts: &'a [ast::Stmt],
            self_name: &str,
            param_types: &HashMap<&str, &'a Expr>,
            out: &mut Vec<(Attribute, Option<&'a Expr>)>,
        ) {
            let self_attr = |target: &Expr| -> Option<String> {
                match target {
                    Expr::Attribute(ast::ExprAttribute { value, attr, .. }) if matches!(value.as_ref(), Expr::Name(n) if n.id.as_str() == self_name) => {
                        Some(attr.to_string())
                    }
                    _ => None,
                }
            };
            let visibility = |name: &str| {
                if name.starts_with('_') && !(name.starts_with("__") && name.ends_with("__")) {
                    Visibility::Private
                } else {
                    Visibility::Public
                }
            };
            for stmt in stmts {
                match stmt {
                    ast::Stmt::AnnAssign(ast::StmtAnnAssign {
                        target, annotation, ..
                    }) => {
                        if let Some(name) = self_attr(target) {
                            let type_annotation = match annotation.as_ref() {
                                Expr::StringLiteral(l) => l.value.to_str().to_string(),
                                other => checker.generator().expr(other),
                            };
                            out.push((
                                Attribute {
                                    visibility: visibility(&name),
                                    name,
                                    type_annotation,
                                },
                                Some(annotation.as_ref()),
                            ));
                        }
                    }
                    ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) => {
                        for name in targets.iter().filter_map(self_attr) {
                            let type_annotation = match value.as_ref() {
                                Expr::Name(n) if param_types.contains_key(n.id.as_str()) => {
                                    match param_types[n.id.as_str()] {
                                        Expr::StringLiteral(l) => l.value.to_str().to_string(),
                                        other => checker.generator().expr(other),
                                    }
                                }
                                other => match ClassDiagram::infer_value_type(other) {
                                    "" => "Any".to_owned(),
                                    inferred => inferred.to_owned(),
                                },
                            };
                            out.push((
                                Attribute {
                                    visibility: visibility(&name),
                                    name,
                                    type_annotation,
                                },
                                None,
                            ));
                        }
                    }
                    ast::Stmt::If(ast::StmtIf {
                        body,
                        elif_else_clauses,
                        ..
                    }) => {
                        walk(checker, body, self_name, param_types, out);
                        for clause in elif_else_clauses {
                            walk(checker, &clause.body, self_name, param_types, out);
                        }
                    }
                    ast::Stmt::With(ast::StmtWith { body, .. })
                    | ast::Stmt::For(ast::StmtFor { body, .. })
                    | ast::Stmt::While(ast::StmtWhile { body, .. }) => {
                        walk(checker, body, self_name, param_types, out);
                    }
                    ast::Stmt::Try(ast::StmtTry {
                        body,
                        handlers,
                        orelse,
                        finalbody,
                        ..
                    }) => {
                        walk(checker, body, self_name, param_types, out);
                        for handler in handlers {
                            let ast::ExceptHandler::ExceptHandler(h) = handler;
                            walk(checker, &h.body, self_name, param_types, out);
                        }
                        walk(checker, orelse, self_name, param_types, out);
                        walk(checker, finalbody, self_name, param_types, out);
                    }
                    _ => {}
                }
            }
        }

        let mut out = Vec::new();
        for stmt in &class.body {
            let ast::Stmt::FunctionDef(func) = stmt else {
                continue;
            };
            if func.name.as_str() != "__init__" {
                continue;
            }
            let Some(first) = func
                .parameters
                .posonlyargs
                .iter()
                .chain(&func.parameters.args)
                .next()
            else {
                continue;
            };
            let param_types: HashMap<&str, &Expr> = func
                .parameters
                .iter_non_variadic_params()
                .filter_map(|p| {
                    p.parameter
                        .annotation
                        .as_deref()
                        .map(|a| (p.parameter.name.as_str(), a))
                })
                .collect();
            walk(
                checker,
                &func.body,
                first.parameter.name.as_str(),
                &param_types,
                &mut out,
            );
        }
        out
    }

    /// Cheap literal-based type inference for simple assignments.
    fn infer_value_type(value: &Expr) -> &'static str {
        match value {
            Expr::BoolOp(_) | Expr::BooleanLiteral(_) => "bool",
            Expr::BinOp(_) | Expr::UnaryOp(_) => "int",
            Expr::Lambda(_) => "Callable",
            Expr::DictComp(_) | Expr::Dict(_) => "dict",
            Expr::Set(_) | Expr::SetComp(_) => "set",
            Expr::FString(_) | Expr::StringLiteral(_) => "str",
            Expr::NoneLiteral(_) => "None",
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
        }
    }

    /// Returns true if the function is a property setter or deleter (e.g. @name.setter, @name.deleter).
    /// These are implementation details and should be omitted from the diagram.
    fn is_property_setter_or_deleter(decorator_list: &[ast::Decorator], fn_name: &str) -> bool {
        decorator_list.iter().any(|decorator| {
            UnqualifiedName::from_expr(&decorator.expression).is_some_and(|name| {
                let segs = name.segments();
                (segs == [fn_name, "setter"]) || (segs == [fn_name, "deleter"])
            })
        })
    }

    #[allow(clippy::too_many_lines)]
    fn process_stmt_to_member(checker: &Checker, stmt: &ast::Stmt) -> Option<ClassMember> {
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
                let annotation_name = match annotation.as_ref() {
                    Expr::StringLiteral(literal) => literal.value.to_str().to_string(),
                    other => checker.generator().expr(other),
                };
                let is_dunder = target_name.starts_with("__") && target_name.ends_with("__");
                let is_private = target_name.starts_with('_') && !is_dunder;

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
                let value_type = Self::infer_value_type(value.as_ref());

                // For now, just handle the first target (typical for enums and simple assignments)
                if let Some(Expr::Name(ast::ExprName { id: target, .. })) = targets.first() {
                    let target_name = target.to_string();
                    let type_annotation = if value_type.is_empty() {
                        "Any"
                    } else {
                        value_type
                    }
                    .to_owned();

                    return Some(ClassMember::Attribute(Attribute {
                        name: target_name,
                        type_annotation,
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
                // Skip property setters and deleters - they're implementation details of the property
                if Self::is_property_setter_or_deleter(decorator_list, name.as_str()) {
                    return None;
                }

                let is_dunder = name.starts_with("__") && name.ends_with("__");
                let is_private = name.starts_with('_') && !is_dunder;
                let is_static = is_staticmethod(decorator_list, checker.semantic());

                // @property getters: show as attributes (read-only) instead of methods
                if is_property(
                    decorator_list,
                    std::iter::empty::<QualifiedName>(),
                    checker.semantic(),
                ) {
                    let return_type = returns.as_ref().map_or_else(
                        || {
                            simple_magic_return_type(name)
                                .map_or_else(|| "Any".to_owned(), String::from)
                        },
                        |target| checker.generator().expr(target.as_ref()),
                    );
                    return Some(ClassMember::Attribute(Attribute {
                        name: name.to_string(),
                        type_annotation: return_type,
                        visibility: if is_private {
                            Visibility::Private
                        } else {
                            Visibility::Public
                        },
                    }));
                }

                let render_expr = |expr: &Expr| checker.generator().expr(expr);
                let mut param_gen = ParameterGenerator::new(&render_expr);
                param_gen.unparse_parameters(parameters);
                let params = param_gen.generate();

                let returns = returns
                    .as_ref()
                    .map(|target| checker.generator().expr(target.as_ref()))
                    .or_else(|| simple_magic_return_type(name).map(String::from));

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

        let qualified_name = checker.semantic().resolve_qualified_name(base);

        if qualified_name.as_ref().is_some_and(|name| {
            matches!(name.segments(), ["typing" | "typing_extensions", "Generic"])
                || matches!(name.segments(), ["" | "builtins", "object"])
                || matches!(
                    name.segments(),
                    ["typing" | "typing_extensions", "NamedTuple" | "TypedDict"]
                )
                || is_abc_qualified_name(name)
                || matches!(
                    name.segments(),
                    ["typing" | "typing_extensions", "Protocol"]
                )
        }) {
            return BaseKind::Skip;
        }

        let base_name = qualified_name.map_or_else(
            || {
                let name = checker.locator().slice(base);
                QualifiedName::user_defined(name).normalize_name()
            },
            |base_name| base_name.normalize_name(),
        );

        // Extract just the base class name without the generic specialization,
        // and quote it if it is not a valid bare Mermaid identifier (e.g. dotted names).
        let plain = base_name
            .split('[')
            .next()
            .unwrap_or(&base_name)
            .trim_matches('`')
            .to_string();
        let display = QualifiedName::user_defined(&plain).normalize_name();

        BaseKind::InheritanceTarget {
            name: display,
            is_stdlib_abstract_or_protocol: detector.is_stdlib_abstract_or_protocol(base),
        }
    }

    /// Add source code to the diagram (for stdin/WASM - uses Python defaults)
    pub fn add_source(&mut self, source: &str) {
        self.add_source_with_options(source, PySourceType::Python, ModuleKind::Module);
    }

    /// Add source code from a file path (infers source type and module kind)
    pub fn add_file(&mut self, source: &str, path: &Path) {
        let source_type = PySourceType::from(path);
        let module_kind = Self::module_kind_for_path(path);
        self.add_source_with_options(source, source_type, module_kind);
    }

    fn add_source_with_options(
        &mut self,
        source: &str,
        source_type: PySourceType,
        module_kind: ModuleKind,
    ) {
        let source_kind = SourceKind::Python {
            code: source.to_owned(),
            is_stub: false,
        };

        let parsed = Self::parse_python(source_kind.source_code(), source_type);
        let mut checker = Self::build_checker(
            &parsed.stylist,
            &parsed.locator,
            &parsed.python_ast,
            module_kind,
        );
        checker.see_imports(&parsed.python_ast);
        self.syntax_errors
            .extend(parsed.syntax_errors.iter().cloned());

        self.add_classes_from_ast(&checker, &parsed.python_ast);
        self.diagram.resolve_references();
        self.diagram.finalize_relation_types();
    }

    fn add_classes_from_ast(&mut self, checker: &Checker, python_ast: &[ast::Stmt]) {
        self.add_classes_in_scope(checker, python_ast, &mut Vec::new());
    }

    /// Walk statements looking for class definitions, descending into class bodies,
    /// function bodies and compound statements. `scope` holds the enclosing
    /// class/function names, outermost first.
    fn add_classes_in_scope<'a>(
        &mut self,
        checker: &Checker,
        stmts: &'a [ast::Stmt],
        scope: &mut Vec<&'a str>,
    ) {
        for stmt in stmts {
            match stmt {
                ast::Stmt::ClassDef(class) => {
                    self.add_class_in_scope(checker, class, scope);
                    scope.push(class.name.as_str());
                    self.add_classes_in_scope(checker, &class.body, scope);
                    scope.pop();
                }
                ast::Stmt::FunctionDef(func) => {
                    scope.push(func.name.as_str());
                    self.add_classes_in_scope(checker, &func.body, scope);
                    scope.pop();
                }
                ast::Stmt::If(ast::StmtIf {
                    body,
                    elif_else_clauses,
                    ..
                }) => {
                    self.add_classes_in_scope(checker, body, scope);
                    for clause in elif_else_clauses {
                        self.add_classes_in_scope(checker, &clause.body, scope);
                    }
                }
                ast::Stmt::With(ast::StmtWith { body, .. })
                | ast::Stmt::For(ast::StmtFor { body, .. })
                | ast::Stmt::While(ast::StmtWhile { body, .. }) => {
                    self.add_classes_in_scope(checker, body, scope);
                }
                ast::Stmt::Try(ast::StmtTry {
                    body,
                    handlers,
                    orelse,
                    finalbody,
                    ..
                }) => {
                    self.add_classes_in_scope(checker, body, scope);
                    for handler in handlers {
                        let ast::ExceptHandler::ExceptHandler(h) = handler;
                        self.add_classes_in_scope(checker, &h.body, scope);
                    }
                    self.add_classes_in_scope(checker, orelse, scope);
                    self.add_classes_in_scope(checker, finalbody, scope);
                }
                _ => {}
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
        let locator = Locator::new(source);
        let syntax_errors = parsed
            .errors()
            .iter()
            .map(|err| {
                let offset = usize::from(err.location.start()).min(source.len());
                let line = source.as_bytes()[..offset]
                    .iter()
                    .filter(|&&b| b == b'\n')
                    .count()
                    + 1;
                format!("line {line}: {}", err.error)
            })
            .collect();
        let python_ast = parsed.into_suite().to_vec();

        ParsedPython {
            python_ast,
            locator,
            stylist,
            syntax_errors,
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
    /// Human-readable syntax errors, e.g. `line 3: Expected an expression`.
    syntax_errors: Vec<String>,
}

#[cfg(test)]
mod tests;
