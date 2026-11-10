from typing import Optional


class Request:
    path: str


class Response:
    status: int


class Middleware:
    pass


class Handler:
    def handle(self, request: Request) -> Response: ...

    # Wrapped, union and container annotations are unwrapped.
    def maybe(self, request: Optional[Request]) -> "list[Response] | None": ...

    # Keyword-only and positional-only parameters count.
    def configure(self, middleware: Middleware, /, *, request: Request) -> None: ...

    @classmethod
    def build(cls, middleware: Middleware) -> "Handler": ...

    @staticmethod
    def parse(raw: str) -> Response: ...

    async def fetch(self, request: Request) -> Response: ...

    # Builtins and unannotated parameters create no edges.
    def plain(self, a: int, b: str, c) -> bool: ...
