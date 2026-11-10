class Outer:
    class Inner:
        value: int

        class Deep:
            flag: bool

    inner: Inner


class Service:
    class Config:
        debug: bool = False

    config: Config
