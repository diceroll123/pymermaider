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
}

criterion_group!(benches, bench_parse_render);
criterion_main!(benches);
