from typing import Generic, TypeVar

T = TypeVar("T")


class Database:
    host: str


class Session:
    def query(self, sql: str) -> None: ...


class Node:
    def link(self, other: "Node") -> "Node":
        return self


class Box(Generic[T]):
    def get(self) -> T:
        ...


class Service:
    db: "Database"

    def connect(self, database: "Database") -> "Session":
        return Session()

    def open_session(self) -> "Session":
        return self.db.session()
