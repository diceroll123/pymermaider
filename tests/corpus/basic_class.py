class Point:
    x: float
    y: float

    def __init__(self, x: float, y: float) -> None:
        self.x = x
        self.y = y

    def scale(self, factor: float) -> None:
        self.x *= factor
        self.y *= factor

    def is_origin(self) -> bool:
        return self.x == 0 and self.y == 0
