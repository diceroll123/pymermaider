from typing import Generic, TypeVar

T = TypeVar("T")


class Box(Generic[T]):
    value: T

    def get(self) -> T:
        return self.value


class Pair[A, B]:
    first: A
    second: B

    def describe(self) -> str:
        return f"{self.first}:{self.second}"
