import abc
from typing import Protocol


class Drawable(Protocol):
    def draw(self) -> None: ...


class Shape(metaclass=abc.ABCMeta):
    @abc.abstractmethod
    def area(self) -> float: ...


class Circle(Shape):
    radius: float

    def area(self) -> float:
        return 3.14 * self.radius ** 2


class Canvas(Drawable):
    width: int

    def draw(self) -> None:
        pass
