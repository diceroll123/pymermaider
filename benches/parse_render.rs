use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pymermaider_wasm::class_diagram::ClassDiagram;

const PYDANTIC_FIXTURE: &str = r#"
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

class User(UserBase):
    id: int
    is_active: bool
    items: list[Item] = []

    class Config:
        orm_mode = True
"#;

const FEATURE_RICH_FIXTURE: &str = r#"
from __future__ import annotations

import abc
import enum
from dataclasses import dataclass, field
from typing import ClassVar, Generic, Protocol, TypeVar, overload

T = TypeVar("T")


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


class Shape(abc.ABC):
    sides: ClassVar[int] = 0

    @abc.abstractmethod
    def area(self) -> float: ...

    @property
    def name(self) -> str:
        return type(self).__name__

    @name.setter
    def name(self, value: str) -> None:
        pass


@dataclass
class Point:
    x: float = 0.0
    y: float = 0.0
    tags: list[str] = field(default_factory=list)


class Circle(Shape):
    sides = 0

    def __init__(self, center: Point, radius: float, color: Color = Color.RED) -> None:
        self.center = center
        self.radius = radius
        self._color = color

    def area(self) -> float:
        return 3.14159 * self.radius ** 2

    def __repr__(self):
        return f"Circle({self.center}, {self.radius})"

    @classmethod
    def unit(cls) -> Circle:
        return cls(Point(), 1.0)

    @staticmethod
    def _validate(radius: float) -> bool:
        return radius > 0


class Drawable(Protocol):
    def draw(self, canvas: Canvas, *, scale: float = 1.0) -> None: ...


class Container(Generic[T]):
    items: list[T]

    def __init__(self, *items: T, **meta: str) -> None:
        self.items = list(items)
        self.__secret = meta

    @overload
    def get(self, index: int) -> T: ...
    @overload
    def get(self, index: slice) -> list[T]: ...
    def get(self, index):
        return self.items[index]

    def __len__(self):
        return len(self.items)


class Canvas(Container[Shape]):
    background: Color
    origin: Point

    async def render(self, shapes: list[Shape], drawer: Drawable | None = None) -> bytes:
        return b""
"#;

/// Generate a large Python source (roughly `target_chars` characters).
fn generate_large_source(target_chars: usize) -> String {
    let mut out = String::from("from typing import Optional\n\n");
    let mut i: usize = 0;
    while out.len() < target_chars {
        let prev = i.saturating_sub(1);
        out.push_str(&format!(
            "class Model{i}(Model{prev}):\n    name: str\n    count: int\n    other: Optional[Model{prev}]\n\n    def __init__(self, name: str, count: int) -> None:\n        self.name = name\n        self.count = count\n\n    def compute(self, x: int, y: int = 2) -> int:\n        return x + y\n\n    @property\n    def label(self) -> str:\n        return self.name\n\n"
        ));
        i += 1;
    }
    out
}

fn render(source: &str) -> Option<String> {
    let mut diagram = ClassDiagram::default();
    diagram.add_source(source);
    diagram.render()
}

fn bench_parse_render(c: &mut Criterion) {
    let large = generate_large_source(50_000);
    c.bench_function("large_file_50k_chars", |b| {
        b.iter(|| render(black_box(&large)));
    });
    c.bench_function("pydantic_fixture", |b| {
        b.iter(|| render(black_box(PYDANTIC_FIXTURE)));
    });
    c.bench_function("feature_rich_fixture", |b| {
        b.iter(|| render(black_box(FEATURE_RICH_FIXTURE)));
    });
    c.bench_function("feature_rich_fixture_hide_private", |b| {
        b.iter(|| {
            let mut diagram = ClassDiagram::default();
            diagram.set_hide_private_members(true);
            diagram.add_source(black_box(FEATURE_RICH_FIXTURE));
            diagram.render()
        });
    });
}

fn bench_phases(c: &mut Criterion) {
    let large = generate_large_source(50_000);

    // Parsing + semantic analysis only (no rendering).
    c.bench_function("analyze_only_large_file_50k_chars", |b| {
        b.iter(|| {
            let mut diagram = ClassDiagram::default();
            diagram.add_source(black_box(&large));
            diagram
        });
    });

    // Rendering only, on a diagram built once outside the measured loop.
    let mut prebuilt = ClassDiagram::default();
    prebuilt.add_source(&large);
    c.bench_function("render_only_large_file_50k_chars", |b| {
        b.iter(|| black_box(&prebuilt).render());
    });
}

fn bench_multi_file(c: &mut Criterion) {
    // Simulates the CLI merging many per-file diagrams into one output.
    let sources: Vec<&str> = std::iter::repeat_n([PYDANTIC_FIXTURE, FEATURE_RICH_FIXTURE], 10)
        .flatten()
        .collect();
    c.bench_function("merge_20_files", |b| {
        b.iter(|| {
            let merged = sources.iter().fold(ClassDiagram::default(), |acc, source| {
                let mut diagram = ClassDiagram::default();
                diagram.add_source(black_box(source));
                acc.merge(diagram)
            });
            merged.render()
        });
    });
}

criterion_group!(benches, bench_parse_render, bench_phases, bench_multi_file);
criterion_main!(benches);
