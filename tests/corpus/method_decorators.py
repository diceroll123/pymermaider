from typing import final, overload


class Worker:
    @classmethod
    def from_config(cls, config: str) -> "Worker":
        return Worker()

    @staticmethod
    def parse(value: int) -> str:
        return str(value)

    @final
    def commit(self) -> None:
        pass

    @overload
    def get(self, key: str) -> int: ...
    @overload
    def get(self, key: int) -> str: ...

    async def fetch(self, url: str) -> bytes:
        return b""
