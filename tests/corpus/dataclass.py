from dataclasses import dataclass, field


@dataclass
class User:
    id: int
    name: str
    tags: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class Coordinate:
    x: float
    y: float
