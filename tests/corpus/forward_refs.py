class Node:
    def __init__(self, value: int) -> None:
        self.value = value
        self.next: "Node" | None = None

    def link(self, other: "Node") -> "Node":
        self.next = other
        return self


class Graph:
    head: "Node"

    def add(self, node: "Node") -> None:
        self.head = node
