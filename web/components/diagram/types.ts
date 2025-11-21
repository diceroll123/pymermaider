export type PyMermaiderClass = {
  processPythonCode(code: string): string;
};

export const DEFAULT_PYTHON_CODE = `class Animal:
    def __init__(self, name: str) -> None:
        self.name = name

    def speak(self) -> str:
        pass

class Dog(Animal):
    def speak(self) -> str:
        return "Woof!"

    def bark(self) -> str:
        return self.speak()

class Cat(Animal):
    def speak(self) -> str:
        return "Meow!"

    def purr(self) -> None:
        print("Purrrr")
`;
