export function isPythonFile(name: string): boolean {
  return name.endsWith(".py") || name.endsWith(".pyi");
}

export function isStubFile(name: string | null | undefined): boolean {
  return !!name && name.endsWith(".pyi");
}
