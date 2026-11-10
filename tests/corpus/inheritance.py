class Animal:
    name: str

    def speak(self) -> str:
        raise NotImplementedError


class Dog(Animal):
    breed: str

    def speak(self) -> str:
        return "woof"


class Fish(Animal):
    def speak(self) -> str:
        return "blub"
