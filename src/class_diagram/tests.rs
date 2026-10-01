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
        + int x
        + int y
        + \_\_init__(self, x: int, y: int) None
        + add(self, x: int, y: int) int
        + subtract(self, x: int, y: int) int
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

    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);

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
        + @classmethod async foo(cls, first, /, *second, kwarg: bool = True, **unpack_this) dict~str, str~
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
        + list~Wheel~ wheels
        + drive(self) None
    }

    Car *-- Engine

    Car *-- Wheel
";

    test_diagram(source, expected_output);
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

    let expected_output = "classDiagram
    class Engine {
        + int horsepower
    }

    class Wheel {
        + int diameter
    }

    class Car {
        + Engine | Wheel part
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
        + list~Item~ items
    }

    class UserCreate {
        + str password
    }

    class `Item.Config` {
        + bool orm_mode
    }

    class `User.Config` {
        + bool orm_mode
    }

    ItemBase --|> `pydantic.BaseModel`

    ItemCreate --|> ItemBase

    Item --|> ItemBase

    UserBase --|> `pydantic.BaseModel`

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
        + @overload \_\_init__(self, x: int, y: int) None
        + @overload \_\_init__(self, x: str, y: str) None
        + \_\_init__(self, x: int | str, y: int | str) None
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
        + \_\_complex__(self) complex
        + \_\_bytes__(self) bytes
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
        + @staticmethod static_method(x: int, y: int) int$
    }
";

    test_diagram(source, expected_output);
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
    let expected_output = "classDiagram
    class Person {
        + str name
    }
";

    test_diagram(source, expected_output);
}

#[test]
fn test_property_no_return_annotation() {
    let source = "
class Thing:
    @property
    def value(self):
        return 42
";
    let expected_output = "classDiagram
    class Thing {
        + Any value
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
        + FancyStorage storage
        + \_\_init__(self, fancy_store: FancyStorage) None
        + insert(self, data) None
    }

    class MemoryStore {
        + insert(self, data) None
    }

    MemoryStore ..|> Store

    FancyStore ..|> Store"#;

    test_diagram(source, expected_output);
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

fn test_diagram(source: &str, expected_output: &str) {
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    let output = diagram.render().unwrap_or_default();
    assert_eq!(output.trim(), expected_output.trim());
}

#[expect(dead_code)]
fn test_diagram_print(source: &str) {
    // for making new tests and debugging :P
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    println!("{}", diagram.render().unwrap_or_default());
    assert_eq!(1, 2);
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

    assert!(
        out.contains("+ engine: Engine") || out.contains("Engine engine"),
        "{out}"
    );
    assert!(!out.contains('"'), "{out}");
    assert!(out.contains("Car *-- Engine"), "{out}");
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

    assert!(out.contains("Car *-- Engine"), "{out}");
    assert!(out.contains("Car *-- Wheel"), "{out}");
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

    assert!(out.contains("+ int base"), "{out}");
    assert!(out.contains("+ str typed_only"), "{out}");
    assert!(out.contains("+ slow(self) None"), "{out}");
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

    assert!(
        out.contains("class Repo ~T~ {\n        <<interface>>"),
        "{out}"
    );
    assert!(
        out.contains("class Multi {\n        <<interface>>"),
        "{out}"
    );
    assert!(!out.contains("Protocol"), "{out}");
    assert!(
        out.contains("Multi ..|> Base") || out.contains("Multi --|> Base"),
        "{out}"
    );
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

    assert!(out.contains("Child ..|> Base"), "{out}");
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

    assert!(out.contains("Item --|> `pydantic.BaseModel`"), "{out}");
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

    assert!(out.contains("class Outer"), "{out}");
    assert!(out.contains("class `Outer.Inner`"), "{out}");
    assert!(out.contains("class `Outer.Inner.Deep`"), "{out}");
    // Bare reference to the nested class resolves to its qualified name
    assert!(out.contains("Outer *-- `Outer.Inner`"), "{out}");
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

    assert!(out.contains("`Outer.Child` --|> `Outer.Base`"), "{out}");
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

    assert!(out.contains("User --|> Config"), "{out}");
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

    assert!(out.contains("class `make.Local`"), "{out}");
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

    assert!(out.contains("Car *-- Engine"), "{out}");
    assert!(out.contains("Car *-- `pathlib.Path`"), "{out}");
    assert!(out.contains("Car *-- `decimal.Decimal`"), "{out}");
}

#[test]
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

    assert!(out.contains("dict~str, list~int~~ mapping"), "{out}");
    assert!(
        out.contains("Literal~#quot;a#quot;, #quot;b#quot;~ mode"),
        "{out}"
    );
    assert!(!out.contains('['), "{out}");
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

    assert!(out.contains("+ str name"), "{out}");
    assert!(out.contains("+ Engine engine"), "{out}");
    assert!(out.contains("- float \\_secret"), "{out}");
    assert!(out.contains("+ bool flag"), "{out}");
    // Declared at class level, so not duplicated by the __init__ assignment
    assert_eq!(out.matches(" wheels\n").count(), 1, "{out}");
    assert!(out.contains("Car *-- Engine"), "{out}");
    assert!(
        out.contains("init__(self, name: str, engine: Engine, wheels: int = 4) None"),
        "{out}"
    );
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

    assert!(
        out.contains("run(self, a, b=2, *, c: str = #quot;x#quot;) None"),
        "{out}"
    );
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

    assert!(out.contains("<<abstract>>"), "{out}");
    assert!(out.contains("run(self) None*"), "{out}");
    assert!(out.contains("@classmethod make(cls)"), "{out}");
    assert!(out.contains("@staticmethod helper() int$"), "{out}");
    assert!(out.contains("@final locked(self) None"), "{out}");
    assert!(out.contains("async fetch(self) bytes"), "{out}");
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

    assert_eq!(out.matches("value").count(), 1, "{out}");
    assert!(out.contains("+ int value"), "{out}");
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

    assert!(out.contains("ClassVar"), "{out}");
    assert!(out.contains("count"), "{out}");
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

    assert!(
        out.contains("run(self, a, /, b, *args, c, **kwargs) None"),
        "{out}"
    );
}
