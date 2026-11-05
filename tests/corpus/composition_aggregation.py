from typing import Optional


class Engine:
    horsepower: int


class Wheel:
    radius: float


class TowHitch:
    capacity: int


class Car:
    engine: Engine
    wheels: list[Wheel]
    tow_hitch: Optional[TowHitch]
