export type PyMermaiderClass = {
  processPythonCode(code: string): string;
};

// File tree types for the explorer
export interface FileNode {
  id: string;           // full path, used as unique key
  name: string;         // display name
  children?: FileNode[];
  content?: string;     // lazy-loaded for files
  isPython?: boolean;   // true for .py files
  childrenCount?: number; // for lazy loading (GitHub)
}

export interface RepoSource {
  type: 'local' | 'github';
  name: string;
  owner?: string;  // for github repos
  repo?: string;   // for github repos
}

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
