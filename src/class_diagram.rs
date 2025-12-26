use crate::ast;
use crate::checker::Checker;
use crate::class_helpers::{ClassDefHelpers, QualifiedNameHelpers};
use crate::class_type_detector::ClassTypeDetector;
use crate::mermaid_renderer::MermaidRenderer;
use crate::parameter_generator::ParameterGenerator;
use crate::renderer::*;
use crate::type_analyzer;
use itertools::Itertools as _;
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
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Represents a class member (attribute or method) during processing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ClassMember {
    Attribute(Attribute),
    Method(MethodSignature),
}

pub struct ClassDiagram {
    diagram: Diagram,
    protocol_classes: HashSet<String>,
    abstract_classes: HashSet<String>,
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
            protocol_classes: HashSet::new(),
            abstract_classes: HashSet::new(),
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

        // Use new modular rendering architecture
        let mut diagram = self.diagram.clone();
        diagram.title = if self.path.is_empty() {
            None
        } else {
            Some(self.path.clone())
        };

        diagram.sort_classes_topologically();

        let renderer = MermaidRenderer::new();
        renderer.render_diagram(&diagram)
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
        let mut composition_types: Vec<String> = vec![];
        for stmt in &class.body {
            if let ast::Stmt::AnnAssign(ast::StmtAnnAssign { annotation, .. }) = stmt {
                if let Some(type_name) =
                    type_analyzer::extract_composition_type(annotation.as_ref(), checker)
                {
                    composition_types.push(type_name);
                }
            }
        }

        // Process class body statements
        let members: Vec<ClassMember> = class
            .body
            .iter()
            .filter_map(|stmt| self.process_stmt_to_member(checker, stmt))
            .unique()
            .collect();

        // Detect class type using ClassTypeDetector
        let detector = ClassTypeDetector::new(checker);
        let class_type = detector.detect_type(class);

        // Track protocols and abstract classes for relationship detection
        if matches!(class_type, ClassType::Interface) {
            self.protocol_classes.insert(class_name.clone());
        }
        if matches!(class_type, ClassType::Abstract) {
            self.abstract_classes.insert(class_name.clone());
        }

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
            // skip if it's a generic type or a built-in type or an ABC or an enum or a Protocol
            let should_skip = type_analyzer::extract_generic_params(base, checker).is_some()
                || checker
                    .semantic()
                    .resolve_qualified_name(base)
                    .is_some_and(|name| {
                        matches!(name.segments(), ["typing", "Generic"])
                            || matches!(name.segments(), ["" | "builtins", "object"])
                            || matches!(name.segments(), ["abc", "ABC" | "ABCMeta"])
                            || matches!(name.segments(), ["typing", "ABC"])
                            || matches!(name.segments(), ["typing", "Protocol"])
                            || matches!(name.segments(), ["typing_extensions", "Protocol"])
                    })
                || class.is_enum(checker.semantic());

            if should_skip {
                continue;
            }

            let base_name = match checker.semantic().resolve_qualified_name(base) {
                Some(base_name) => base_name.normalize_name(),
                None => {
                    let name = checker.locator().slice(base);
                    QualifiedName::user_defined(name).normalize_name()
                }
            };

            // Extract just the base class name without the generic specialization
            let base_display = if base_name.contains('[') {
                base_name
                    .split('[')
                    .next()
                    .unwrap_or(&base_name)
                    .to_string()
            } else {
                base_name
            }
            .trim_matches('`')
            .to_string();

            // Check if the base class is abstract or a protocol (either built-in or user-defined)
            let base_is_abstract_or_protocol = detector.is_base_abstract_or_protocol(
                &base_display,
                base,
                &self.protocol_classes,
                &self.abstract_classes,
            );

            let rel = RelationshipEdge {
                from: class_name.clone(),
                to: base_display,
                relation_type: if base_is_abstract_or_protocol {
                    RelationType::Implementation
                } else {
                    RelationType::Inheritance
                },
            };
            self.diagram.add_relationship(rel);
        }

        // Add composition relationships
        for comp_type in composition_types.iter().unique() {
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

    pub fn add_to_diagram(&mut self, source: String, file: &PathBuf) {
        let source_type = PySourceType::from(file);
        let source_kind = SourceKind::Python(source);

        let locator = Locator::new(source_kind.source_code());

        let parsed = parse_unchecked_source(source_kind.source_code(), source_type);

        let stylist = Stylist::from_tokens(parsed.tokens(), source_kind.source_code());

        let python_ast = parsed.into_suite();

        let kind = if file.ends_with("__init__.py") {
            ModuleKind::Package
        } else {
            ModuleKind::Module
        };

        let module = Module {
            kind,
            source: ModuleSource::File(Path::new(file)),
            python_ast: &python_ast,
            name: None,
        };
        let semantic = SemanticModel::new(&[], Path::new(file), module);
        let mut checker = Checker::new(&stylist, &locator, semantic);
        checker.see_imports(&python_ast);

        for stmt in &python_ast {
            if let ast::Stmt::ClassDef(class) = stmt {
                // we only care about class definitions
                self.add_class(&checker, class, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_diagram_basic() {
        let source = "
class TestClass:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y
    def add(self, x: int, y: int) -> int:
        return x + y
    def subtract(self, x: int, y: int) -> int:
        return x - y
";

        let expected_output = r"classDiagram
    class TestClass {
        - \_\_init__(self, x, y) None
        + add(self, x, y) int
        + subtract(self, x, y) int
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_raw_mermaid_has_no_fences() {
        let source = r#"
class TestClass:
    def add(self, x: int, y: int) -> int:
        return x + y
"#;

        let mut diagram = ClassDiagram::new();
        diagram.path = "example.py".to_string();
        diagram.add_to_diagram(source.to_string(), &PathBuf::from("example.py"));

        let raw = diagram.render().unwrap_or_default();

        assert!(!raw.contains("```mermaid"));
        assert!(raw.contains("classDiagram"));
        assert!(raw.contains("class TestClass"));
    }

    #[test]
    fn test_class_diagram_generic_class() {
        let source = "
class Thing[T]: ...
";

        let expected_output = r#"classDiagram
    class Thing ~T~"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_inner_class() {
        let source = "
class Thing(Inner[T]): ...
";

        let expected_output = r#"classDiagram
    class Thing

    Thing --|> Inner"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic() {
        let source = r#"
from typing import TypeVar, Generic
from abc import ABC
FancyType = TypeVar("FancyType")
class Thing(ABC, Generic[FancyType]): ...
"#;

        let expected_output = r#"classDiagram
    class Thing ~FancyType~ {
        <<abstract>>
    }"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_class_multiple() {
        let source = "
class Thing[T, U, V]: ...
";

        let expected_output = r#"classDiagram
    class Thing ~T, U, V~"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_final() {
        let source = "
from typing import final
@final
class Thing: ...
";

        let expected_output = "classDiagram
    class Thing {
        <<final>>
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_ellipsis() {
        let source = "
class Thing: ...
";

        let expected_output = "classDiagram
    class Thing
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_complex() {
        // this tests async, classmethod, args, return type
        let source = "
class Thing:
    @classmethod
    async def foo(cls, first, /, *second, kwarg: bool = True, **unpack_this) -> dict[str, str]: ...
";

        let expected_output = "classDiagram
    class Thing {
        + @classmethod async foo(cls, first, /, *second, kwarg, **unpack_this) dict[str, str]
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_dataclass() {
        let source = "
from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int

    def greet(self) -> str:
        return f'Hello, I am {self.name}'
";

        let expected_output = "classDiagram
    class Person {
        <<dataclass>>
        + str name
        + int age
        + greet(self) str
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_protocol() {
        let source = "
from typing import Protocol

class Drawable(Protocol):
    def draw(self) -> None:
        ...

class Circle(Drawable):
    def draw(self) -> None:
        pass
";

        let expected_output = "classDiagram
    class Drawable {
        <<interface>>
        + draw(self) None
    }

    class Circle {
        + draw(self) None
    }

    Circle ..|> Drawable
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_composition_relationships() {
        let source = "
class Engine:
    horsepower: int

class Wheel:
    diameter: int

class Car:
    engine: Engine
    wheels: list[Wheel]

    def drive(self) -> None:
        pass
";

        let expected_output = "classDiagram
    class Engine {
        + int horsepower
    }

    class Wheel {
        + int diameter
    }

    class Car {
        + Engine engine
        + list[Wheel] wheels
        + drive(self) None
    }

    Car *-- Engine

    Car *-- Wheel
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_pydantic_example() {
        let source = "
from pydantic import BaseModel


class ItemBase(BaseModel):
    title: str
    description: str | None = None


class ItemCreate(ItemBase):
    pass


class Item(ItemBase):
    id: int
    owner_id: int

    class Config:
        orm_mode = True


class UserBase(BaseModel):
    email: str


class UserCreate(UserBase):
    password: str


class User(UserBase):
    id: int
    is_active: bool
    items: list[Item] = []

    class Config:
        orm_mode = True
";

        let expected_output = "classDiagram
    class ItemBase {
        + str title
        + str | None description
    }

    class Item {
        + int id
        + int owner_id
    }

    class ItemCreate

    class UserBase {
        + str email
    }

    class User {
        + int id
        + bool is_active
        + list[Item] items
    }

    class UserCreate {
        + str password
    }

    ItemBase --|> pydantic.BaseModel

    ItemCreate --|> ItemBase

    Item --|> ItemBase

    UserBase --|> pydantic.BaseModel

    UserCreate --|> UserBase

    User --|> UserBase

    User *-- Item
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_unique_overloads() {
        let source = "
from typing import overload
class Thing:
    @overload
    def __init__(self, x: int, y: int) -> None: ...

    @overload
    def __init__(self, x: str, y: str) -> None: ...

    def __init__(self, x: int | str, y: int | str) -> None: ...
";

        let expected_output = r"classDiagram
    class Thing {
        - @overload \_\_init__(self, x, y) None
        - \_\_init__(self, x, y) None
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_object_base() {
        let source = "
class Thing(object): ...
";

        let expected_output = "classDiagram
    class Thing
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_dundermagic_infer() {
        let source = "
class Thing:
    def __complex__(self): ...
    def __bytes__(self): ...
";

        let expected_output = r"classDiagram
    class Thing {
        - \_\_complex__(self) complex
        - \_\_bytes__(self) bytes
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_notimplemented() {
        let source = "
class Thing:
    def do_thing(self):
        raise NotImplementedError
";

        let expected_output = "classDiagram
    class Thing {
        + do_thing(self)
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_abstract_base_class() {
        let source = r#"
from abc import ABC, abstractmethod
class Thing(ABC):
    @abstractmethod
    def do_thing(self) -> None:
        """Must be implemented by subclasses"""
        pass
"#;
        let expected_output = "classDiagram
    class Thing {
        <<abstract>>
        + do_thing(self) None*
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_enum() {
        let source = "
from enum import Enum
class Color(Enum):
    RED = 1
    GREEN = 2
    BLUE = 3
";

        let expected_output = "classDiagram
    class Color {
        <<enumeration>>
        + int RED
        + int GREEN
        + int BLUE
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_staticmethod() {
        let source = "
class Thing:
    @staticmethod
    def static_method(x: int, y: int) -> int:
        return x + y
";
        let expected_output = "classDiagram
    class Thing {
        + @staticmethod static_method(x, y) int$
    }
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_concrete_generic_base() {
        let source = r#"
from typing import TypeVar, Generic
IndexType = TypeVar("IndexType")

class Store(Generic[IndexType]):
    def insert(self, data) -> None:
        pass

class MemoryStore(Store[int]):
    def insert(self, data) -> None:
        self.storage.append(data)
"#;

        let expected_output = r#"classDiagram
    class Store ~IndexType~ {
        + insert(self, data) None
    }

    class MemoryStore {
        + insert(self, data) None
    }

    MemoryStore --|> Store"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_abstract_generic_inheritance() {
        let source = r#"
from typing import TypeVar, Generic
from abc import ABC, abstractmethod
IndexType = TypeVar("IndexType")

class Store(ABC, Generic[IndexType]):
    @abstractmethod
    def insert(self, data) -> None:
        pass

class MemoryStore(Store[int]):
    def insert(self, data) -> None:
        self.storage.append(data)
"#;

        let expected_output = r#"classDiagram
    class Store ~IndexType~ {
        <<abstract>>
        + insert(self, data) None*
    }

    class MemoryStore {
        + insert(self, data) None
    }

    MemoryStore ..|> Store"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_full_generics_example() {
        let source = r#"
from typing import TypeVar, Generic
from abc import ABC, abstractmethod
from datetime import datetime

IndexType = TypeVar("IndexType")
FancyStorage = TypeVar("FancyStorage")

class Store(ABC, Generic[IndexType]):
    @abstractmethod
    def insert(self, data) -> None:
        pass

class MemoryStore(Store[datetime]):
    def insert(self, data) -> None:
        self.storage.append(data)

class FancyStore(Store[datetime], Generic[FancyStorage]):
    def __init__(self, fancy_store: FancyStorage) -> None:
        self.storage = fancy_store

    def insert(self, data) -> None:
        self.storage.insert(data)
"#;

        let expected_output = r#"classDiagram
    class Store ~IndexType~ {
        <<abstract>>
        + insert(self, data) None*
    }

    class FancyStore ~FancyStorage~ {
        - \_\_init__(self, fancy_store) None
        + insert(self, data) None
    }

    class MemoryStore {
        + insert(self, data) None
    }

    MemoryStore ..|> Store

    FancyStore ..|> Store"#;

        test_diagram(source, expected_output);
    }

    fn test_diagram(source: &str, expected_output: &str) {
        let mut diagram = ClassDiagram::new();
        diagram.add_to_diagram(source.to_owned(), &PathBuf::from("test.py"));
        let output = diagram.render().unwrap_or_default();
        assert_eq!(output.trim(), expected_output.trim());
    }

    #[expect(dead_code)]
    fn test_diagram_print(source: &str) {
        // for making new tests and debugging :P
        let mut diagram = ClassDiagram::new();
        diagram.add_to_diagram(source.to_owned(), &PathBuf::from("test.py"));
        println!("{}", diagram.render().unwrap_or_default());
        assert_eq!(1, 2);
    }
}
