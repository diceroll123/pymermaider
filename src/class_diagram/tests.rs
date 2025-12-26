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
