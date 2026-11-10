class Database:
    host: str


class Cache:
    key: str


class Logger:
    level: str


class Base:
    def run(self) -> None: ...


class Worker(Base):
    # Base is already covered by inheritance, so no dependency edge.
    def clone_base(self, other: Base) -> Base:
        return other


class Repository:
    # Database is already covered by composition, so no dependency edge.
    db: Database

    def reconnect(self, db: Database) -> Database:
        return db

    # Cache is only referenced by a method, so it stays a dependency.
    def warm(self, cache: Cache) -> None: ...


class Service:
    # Defined twice: the dependency from the first definition is covered by the
    # composition from the second one, regardless of definition order.
    def connect(self, db: Database) -> None: ...

    def log(self, logger: Logger) -> None: ...


class Service:
    db: Database
