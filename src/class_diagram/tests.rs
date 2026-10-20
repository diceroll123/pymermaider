use super::*;

macro_rules! assert_diagram {
    ($source:expr) => {
        insta::assert_snapshot!(render_source($source).trim())
    };
}

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

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_generic_class() {
    let source = "
class Thing[T]: ...
";

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_generic_inner_class() {
    let source = "
class Thing(Inner[T]): ...
";

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_generic() {
    let source = r#"
from typing import TypeVar, Generic
from abc import ABC
FancyType = TypeVar("FancyType")
class Thing(ABC, Generic[FancyType]): ...
"#;

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_generic_class_multiple() {
    let source = "
class Thing[T, U, V]: ...
";

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_final() {
    let source = "
from typing import final
@final
class Thing: ...
";

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_ellipsis() {
    let source = "
class Thing: ...
";

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_complex() {
    // this tests async, classmethod, args, return type
    let source = "
class Thing:
    @classmethod
    async def foo(cls, first, /, *second, kwarg: bool = True, **unpack_this) -> dict[str, str]: ...
";

    assert_diagram!(source);
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

    assert_diagram!(source);
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

    assert_diagram!(source);
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

    assert_diagram!(source);
}

#[test]
fn test_extended_builtins_do_not_create_relationships() {
    let source = "
class Car:
    payload: bytearray
    ratio: complex
    window: slice
    steps: range
";

    assert_diagram!(source);
}

#[test]
fn test_composition_relationships_union_types() {
    let source = "
class Engine:
    horsepower: int

class Wheel:
    diameter: int

class Car:
    part: Engine | Wheel
";

    assert_diagram!(source);
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

    assert_diagram!(source);
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

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_object_base() {
    let source = "
class Thing(object): ...
";

    assert_diagram!(source);
}

#[test]
fn test_class_diagram_dundermagic_infer() {
    let source = "
class Thing:
    def __complex__(self): ...
    def __bytes__(self): ...
";

    assert_diagram!(source);
}

#[test]
fn test_notimplemented() {
    let source = "
class Thing:
    def do_thing(self):
        raise NotImplementedError
";

    assert_diagram!(source);
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

    assert_diagram!(source);
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

    assert_diagram!(source);
}

#[test]
fn test_staticmethod() {
    let source = "
class Thing:
    @staticmethod
    def static_method(x: int, y: int) -> int:
        return x + y
";

    assert_diagram!(source);
}

#[test]
fn test_property_as_attribute() {
    let source = "
class Person:
    @property
    def name(self) -> str:
        return self._name

    @name.setter
    def name(self, value: str) -> None:
        self._name = value

    @name.deleter
    def name(self) -> None:
        del self._name
";

    assert_diagram!(source);
}

#[test]
fn test_property_no_return_annotation() {
    let source = "
class Thing:
    @property
    def value(self):
        return 42
";

    assert_diagram!(source);
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

    assert_diagram!(source);
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

    assert_diagram!(source);
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

    assert_diagram!(source);
}

#[test]
fn test_non_default_direction_emitted() {
    use crate::render::mermaid_renderer::RenderOptions;
    use crate::render::renderer::DiagramDirection;

    let source = "class Thing: ...";
    let expected = "classDiagram
    direction LR

    class Thing
";
    let options = RenderOptions {
        direction: DiagramDirection::LR,
        hide_private_members: false,
    };
    let mut diagram = ClassDiagram::new(options);
    diagram.add_source(source);
    let output = diagram.render().unwrap_or_default();
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn test_hide_private_members() {
    use crate::render::mermaid_renderer::RenderOptions;

    let source = "
class Foo:
    x: int
    _private: str
    def bar(self) -> None: ...
    def _helper(self) -> None: ...
";
    let mut diagram = ClassDiagram::new(RenderOptions::default());
    diagram.add_source(source);
    let with_private = diagram.render().unwrap_or_default();
    assert!(
        with_private.contains("_private"),
        "private attr should appear when not hidden; got: {with_private}"
    );
    assert!(
        with_private.contains("_helper"),
        "private method should appear when not hidden; got: {with_private}"
    );

    diagram.set_hide_private_members(true);
    let without_private = diagram.render().unwrap_or_default();
    assert!(
        without_private.contains("+ int x"),
        "public attr should appear"
    );
    assert!(
        without_private.contains("+ bar(self)"),
        "public method should appear"
    );
    assert!(
        !without_private.contains("_private"),
        "private attr should be hidden; got: {without_private}"
    );
    assert!(
        !without_private.contains("_helper"),
        "private method should be hidden; got: {without_private}"
    );
}

#[test]
fn test_aggregation_vs_composition() {
    let source = r#"
class Engine:
    power: int

class Car:
    engine: Engine
    engines: list[Engine]
"#;
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let result = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(result);
}

#[test]
fn test_list_is_aggregation() {
    let source = r#"
class Engine:
    power: int

class Car:
    engines: list[Engine]
"#;
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let result = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(result);
}

#[test]
fn test_optional_is_aggregation() {
    let source = r#"
from typing import Optional

class Engine:
    power: int

class Car:
    engine: Optional[Engine]
"#;
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let result = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(result);
}

#[test]
fn test_union_with_none_is_aggregation() {
    let source = r#"
class Engine:
    power: int

class Car:
    engine: Engine | None
"#;
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let result = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(result);
}

fn render_source(source: &str) -> String {
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    diagram.render().unwrap_or_default()
}

#[test]
fn test_literal_annotation_creates_no_relationship() {
    let source = r#"
from typing import Literal

class Car:
    mode: Literal["sport", "eco"]
"#;
    let out = render_source(source);
    insta::assert_snapshot!(out);
}

#[test]
fn test_annotated_only_first_arg_is_a_type() {
    let source = r#"
from typing import Annotated

class Engine:
    pass

class Car:
    engine: Annotated[Engine, "metadata"]
"#;
    let out = render_source(source);
    insta::assert_snapshot!(out);
}

#[test]
fn test_callable_annotation_relationships() {
    let source = r"
from typing import Callable

class Request:
    pass

class Response:
    pass

class Handler:
    callback: Callable[[Request], Response]
";
    let out = render_source(source);
    insta::assert_snapshot!(out);
}

#[test]
fn test_binop_and_boolop_value_types() {
    let source = r#"
class Config:
    ratio = 1 / 2
    total = 1 + 2
    name = "a" + "b"
    mixed = 1 + 2.0
    fallback = None or "x"
    flag = not True
"#;
    // Exercise the syntactic path; ty would refine the `Any` cases.
    let mut diagram = ClassDiagram::default();
    #[cfg(feature = "infer")]
    diagram.set_infer(false);
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_string_annotation_forward_reference() {
    let source = r#"
class Engine:
    pass

class Car:
    engine: "Engine"
"#;

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_optional_union_composition() {
    let source = r"
class Engine:
    pass

class Wheel:
    pass

class Car:
    part: Optional[Engine | Wheel]
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_syntax_errors_are_collected() {
    let mut diagram = ClassDiagram::default();
    diagram.add_source("class A:\n    x: int\n\ndef broken(:\n");

    assert!(!diagram.syntax_errors().is_empty());
    assert!(diagram.syntax_errors()[0].starts_with("line 4"));
    assert!(diagram.render().unwrap_or_default().contains("class A"));

    let mut ok = ClassDiagram::default();
    ok.add_source("class B: ...\n");
    assert!(ok.syntax_errors().is_empty());
}

#[test]
fn test_members_in_conditional_class_body_blocks() {
    let source = r"
class Thing:
    base: int

    if TYPE_CHECKING:
        typed_only: str

    try:
        from fast import speed as speed_impl
    except ImportError:
        def slow(self) -> None: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_generic_protocol_is_interface_without_phantom_edge() {
    let source = r"
from typing import Protocol, TypeVar

T = TypeVar('T')

class Base:
    pass

class Repo(Protocol[T]):
    def get(self, key: str) -> T: ...

class Multi(Protocol, Base):
    def run(self) -> None: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_forward_defined_abstract_base_is_implementation() {
    let source = r"
from abc import ABC, abstractmethod

class Child(Base):
    pass

class Base(ABC):
    @abstractmethod
    def run(self) -> None: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_dotted_base_is_backticked() {
    let source = r"
import pydantic

class Item(pydantic.BaseModel):
    pass
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_nested_classes_use_qualified_names() {
    let source = r"
class Outer:
    class Inner:
        x: int

        class Deep:
            y: int

    inner: Inner
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();

    // Bare reference to the nested class resolves to its qualified name
    insta::assert_snapshot!(out);
}

#[test]
fn test_nested_class_as_base_resolves_to_qualified_name() {
    let source = r"
class Outer:
    class Base:
        pass

    class Child(Base):
        pass
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_top_level_class_wins_over_nested_with_same_name() {
    let source = r"
class Config:
    pass

class Item:
    class Config:
        pass

class User(Config):
    pass
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_class_defined_in_function_body() {
    let source = r"
def make():
    class Local:
        pass
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_imported_composition_types_are_kept_and_qualified() {
    let source = r"
from pathlib import Path
from decimal import Decimal

class Engine:
    pass

class Car:
    engine: Engine
    home: Path
    price: Decimal
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_special_characters_in_annotations_are_escaped() {
    let source = r#"
class Config:
    handler: Callable[[int], str]
    mapping: dict[str, list[int]]
    mode: Literal["a", "b"]
"#;

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_instance_attributes_from_init() {
    let source = r#"
class Engine:
    pass

class Car:
    wheels: int

    def __init__(self, name: str, engine: "Engine", wheels: int = 4) -> None:
        self.name = name
        self.engine: Engine = engine
        self._secret = 1.5
        self.wheels = wheels
        if name:
            self.flag = True
"#;

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();

    insta::assert_snapshot!(out);
}

#[test]
fn test_parameter_defaults_without_annotations() {
    let source = r#"
class Thing:
    def run(self, a, b=2, *, c: str = "x") -> None:
        pass
"#;

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_method_kinds_and_decorators() {
    let source = r"
from abc import ABC, abstractmethod
from typing import final

class Base(ABC):
    @abstractmethod
    def run(self) -> None: ...

    @classmethod
    def make(cls) -> 'Base': ...

    @staticmethod
    def helper() -> int: ...

    @final
    def locked(self) -> None: ...

    async def fetch(self) -> bytes: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_property_setter_and_deleter_are_omitted() {
    let source = r"
class Thing:
    @property
    def value(self) -> int: ...

    @value.setter
    def value(self, v: int) -> None: ...

    @value.deleter
    def value(self) -> None: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();

    insta::assert_snapshot!(out);
}

#[test]
fn test_classvar_attribute_is_listed() {
    let source = r"
from typing import ClassVar

class Thing:
    count: ClassVar[int]
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_keyword_only_and_variadic_parameters() {
    let source = r"
class Thing:
    def run(self, a, /, b, *args, c, **kwargs) -> None: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_namedtuple_and_typeddict_have_stereotypes_without_phantom_edges() {
    let source = r"
from typing import NamedTuple, TypedDict

class Point(NamedTuple):
    x: int
    y: int

class Movie(TypedDict):
    title: str
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_abstract_dataclass_keeps_abstract_stereotype() {
    let source = r"
from abc import ABC
from dataclasses import dataclass

@dataclass
class Shape(ABC):
    name: str
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_attrs_classes_are_dataclasses() {
    let source = r"
import attrs

@attrs.define
class Point:
    x: int
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();
    insta::assert_snapshot!(out);
}

#[test]
fn test_protected_and_private_visibility() {
    let source = r"
class Thing:
    _protected: int
    __private: int
    __dunder__: int
    public: int

    def _helper(self) -> None: ...
    def __mangled(self) -> None: ...
";

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let out = diagram.render().unwrap_or_default();

    insta::assert_snapshot!("visible", out);

    // --hide-private-members hides both protected and private members
    let mut hidden = ClassDiagram::default();
    hidden.set_hide_private_members(true);
    hidden.add_source(source);
    let out = hidden.render().unwrap_or_default();
    insta::assert_snapshot!("hidden", out);
}

#[cfg(feature = "infer")]
mod infer {
    use super::*;

    #[test]
    fn test_constructor_call_attribute_gets_type_and_composition() {
        let source = "
class Database:
    pass

class Service:
    def __init__(self):
        self.db = Database()
";
        let out = render_source(source);
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_no_infer_keeps_syntactic_output() {
        let source = "
class Database:
    pass

class Service:
    def __init__(self):
        self.db = Database()
";
        let mut diagram = ClassDiagram::default();
        diagram.set_infer(false);
        diagram.add_source(source);
        let out = diagram.render().unwrap_or_default();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_optional_and_container_inference_is_aggregation() {
        let source = "
class Wheel:
    pass

class Car:
    def __init__(self):
        self.wheels = [Wheel(), Wheel()]
        self.spare = Wheel() if self else None
";
        let out = render_source(source);
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_class_level_assignment_is_inferred() {
        let source = "
class Config:
    pass

class App:
    config = Config()
";
        let out = render_source(source);
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_unknown_types_stay_any() {
        let source = "
class Thing:
    def __init__(self, data):
        self.value = undefined_function()
";
        let out = render_source(source);
        insta::assert_snapshot!(out);
    }

    // --- Lazy-gate tests -------------------------------------------------
    // The gate decides whether building the (expensive) ty project is worth it.
    // It must be a superset of the assignments `refine_with_ty` could refine:
    // whenever it is false, output with and without inference must be identical.

    fn gate(source: &str) -> bool {
        let parsed = ClassDiagram::parse_python(source, crate::ast::PySourceType::Python);
        ClassDiagram::needs_ty_inference(&parsed.python_ast)
    }

    fn render_with_infer(source: &str, infer: bool) -> String {
        let mut diagram = ClassDiagram::default();
        diagram.set_infer(infer);
        diagram.add_source(source);
        diagram.render().unwrap_or_default()
    }

    #[test]
    fn gate_false_sources_render_identically_with_and_without_infer() {
        let sources: &[&str] = &[
            // Literal values have known syntactic types.
            "
class A:
    x = 5
    y = 'hi'
    def __init__(self):
        self.a = True
        self.b = None
",
            // Names bound to annotated parameters keep the parameter's type.
            "
class B:
    def __init__(self, value: str, n: int):
        self.value = value
        self.n = n
",
            // No assignments at all.
            "
class C:
    def do(self) -> None: ...
",
            // Known types nested in compound statements.
            "
class D:
    if True:
        flag = 1
    def __init__(self):
        try:
            self.x = 2
        except Exception:
            pass
",
        ];
        for source in sources {
            assert!(!gate(source), "gate should be false:\n{source}");
            assert_eq!(
                render_with_infer(source, true),
                render_with_infer(source, false)
            );
        }
    }

    #[test]
    fn gate_true_sources_get_ty_inference() {
        let sources: &[&str] = &[
            // Call values in __init__.
            "
class Database: pass

class Service:
    def __init__(self):
        self.db = Database()
",
            // Call values at class level.
            "
class Config: pass

class App:
    config = Config()
",
            // Bare container literals that ty can parametrize.
            "
class Wheel: pass

class Car:
    def __init__(self):
        self.wheels = [Wheel()]
",
            // Class nested in a function.
            "
class Database: pass

def factory():
    class Service:
        def __init__(self):
            self.db = Database()
",
            // Class-level assignment in an if body.
            "
class Config: pass

class App:
    if True:
        config = Config()
",
            // self-assignment in a try body inside __init__.
            "
class Database: pass

class Service:
    def __init__(self):
        try:
            self.db = Database()
        except Exception:
            pass
",
        ];
        for source in sources {
            assert!(gate(source), "gate should be true:\n{source}");
            assert_ne!(
                render_with_infer(source, true),
                render_with_infer(source, false)
            );
        }
    }

    #[test]
    fn gate_true_even_when_ty_finds_nothing() {
        // The gate is a superset: it fires whenever ty *could* be consulted, even if
        // inference ends up with nothing to say. Output is the same either way.
        let sources: &[&str] = &[
            "
class Thing:
    def __init__(self, data):
        self.value = undefined_function()
",
            "
class Thing:
    def __init__(self, data):
        self.value = data
",
        ];
        for source in sources {
            assert!(gate(source), "gate should be true:\n{source}");
            let out = render_with_infer(source, true);
            assert!(out.contains("Any value"), "{out}");
        }
    }
}

#[cfg(feature = "infer")]
#[test]
fn test_container_literal_is_parametrized_by_ty() {
    let source = "
class Wheel:
    pass

class Car:
    def __init__(self):
        self.wheels = [Wheel()]
        self.empty = []
";
    let out = render_source(source);
    // `list[Unknown]` is not informative, so the syntactic `list` is kept.
    insta::assert_snapshot!(out);
}

#[test]
fn test_render_source_with_options_defaults_have_no_title_or_direction() {
    let source = "class A:\n    pass\n";
    let out =
        render_source_with_options(source, DiagramDirection::default(), false, true, None, None)
            .unwrap();
    assert!(out.starts_with("classDiagram\n"), "{out}");
    assert!(!out.contains("title:"));
    assert!(!out.contains("direction "));
}

#[test]
fn test_render_source_with_options_renders_title_and_direction() {
    let source = "class A:\n    pass\n";
    let out = render_source_with_options(
        source,
        DiagramDirection::LR,
        false,
        true,
        Some("My Title"),
        None,
    )
    .unwrap();
    assert!(
        out.starts_with("---\ntitle: My Title\n---\nclassDiagram\n    direction LR\n"),
        "{out}"
    );
}

#[test]
fn test_render_source_with_options_no_classes_returns_none() {
    assert!(render_source_with_options(
        "x = 1\n",
        DiagramDirection::default(),
        false,
        true,
        None,
        None,
    )
    .is_none());
}
