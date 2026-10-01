use pymermaider_wasm::class_diagram::ClassDiagram;
use pymermaider_wasm::RenderOptions;
use std::path::Path;

fn main() {
    divan::main();
}

const SMALL_CLASS: &str = r#"
class Animal:
    name: str
    age: int

    def __init__(self, name: str, age: int) -> None:
        self.name = name
        self.age = age

    def speak(self) -> str:
        return "..."

    def _private(self) -> None:
        pass
"#;

const RELATIONSHIPS: &str = r#"
from typing import Protocol, Optional

class Drawable(Protocol):
    def draw(self) -> None: ...

class Point:
    x: float
    y: float

class Shape:
    origin: Point

    class Style:
        color: str

    def area(self) -> float:
        return 0.0

class Circle(Shape, Drawable):
    radius: float
    style: Optional["Shape.Style"]

    def draw(self) -> None:
        pass

class Canvas:
    shapes: list[Shape]
    background: Optional[Circle]

    def add(self, shape: Shape) -> None:
        self.shapes.append(shape)
"#;

fn generate_module(n: usize) -> String {
    let mut src = String::from("class Base:\n    id: int\n\n");
    for i in 0..n {
        src.push_str(&format!(
            "class Item{i}(Base):\n    value: int\n    other: Base\n\n    def get(self, x: int) -> int:\n        return x\n\n    def _hidden(self) -> None:\n        pass\n\n"
        ));
    }
    src
}

fn build(source: &str) -> ClassDiagram {
    let mut diagram = ClassDiagram::new(RenderOptions::default());
    diagram.add_source(source);
    diagram
}

#[divan::bench]
fn bench_small_class() {
    divan::black_box(build(divan::black_box(SMALL_CLASS)).render());
}

#[divan::bench]
fn bench_relationships() {
    divan::black_box(build(divan::black_box(RELATIONSHIPS)).render());
}

#[divan::bench(args = [10, 100])]
fn bench_large_module(bencher: divan::Bencher, n: usize) {
    bencher
        .with_inputs(|| generate_module(n))
        .bench_refs(|src| divan::black_box(build(src).render()));
}

#[divan::bench]
fn bench_render_only(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| build(RELATIONSHIPS))
        .bench_refs(|diagram| divan::black_box(diagram.render()));
}

#[divan::bench]
fn bench_merge() {
    let mut a = ClassDiagram::new(RenderOptions::default());
    a.add_file(divan::black_box(SMALL_CLASS), Path::new("a.py"));
    let mut b = ClassDiagram::new(RenderOptions::default());
    b.add_file(divan::black_box(RELATIONSHIPS), Path::new("b.py"));
    divan::black_box(a.merge(b).render());
}
