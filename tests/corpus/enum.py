from enum import Enum, auto


class Color(Enum):
    RED = 1
    GREEN = 2
    BLUE = auto()


class Direction(str, Enum):
    NORTH = "N"
    SOUTH = "S"
