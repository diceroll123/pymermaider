# Changelog

## [Unreleased]

### Added
- `--include` flag to include only files matching the given patterns (complements `--exclude`).
- `--direction` flag to set class diagram direction (TB, BT, LR, RL).
- `--no-title` flag to omit the title from diagram output.
- `--hide-private-members` flag to omit private members from class diagrams.
- Snapshot tests (`insta`) over a Python corpus in `tests/corpus/`, pinning exact Mermaid output for common features (inheritance, protocols/ABCs, enums, dataclasses, generics, nested classes, composition/aggregation, decorators, forward references).
- Python API via a native (pyo3) module: `pymermaider.generate(source)` returns the raw Mermaid diagram and `pymermaider.to_markdown(source)` wraps it in a fenced ```` ```mermaid ```` block. Both accept `direction`, `hide_private_members`, `show_title`, `title` and `python_version` options.
- Web app: Export menu in the diagram editor header to copy the Mermaid code or download the diagram as `.mmd`, Markdown (`.md`), SVG, or PNG.

### Changed
- Documented type inference differences between the native CLI (project-wide) and the web/WASM build (single-file, cross-module imports unresolved).
- Property decorator handling: `@property` getters are now shown as attributes (e.g. `+ str name`) instead of methods. Property setters and deleters (`@name.setter`, `@name.deleter`) are omitted from the diagram.
- The PyPI wheel now ships the Python API alongside the CLI. The `pymermaider` executable is a console script entry point into the native module (no separate binary in the wheel); `cargo install --git https://github.com/diceroll123/pymermaider pymermaider` still builds a standalone native binary.
- Dropped Python 3.8 support; the wheel now targets CPython 3.9+ via the stable ABI (abi3).

### Fixed
- Expanded the built-in type list used to filter composition/relationship edges with `bytearray`, `frozenset`, `complex`, `memoryview`, `object`, `type`, `slice`, and `range`, so annotations like `x: range` no longer create spurious relationship edges.

### Fixed
- Type-inference literal normalization: replaced a byte-index string slice with bounds-safe `str::get` (no behavior change).

## [0.1.6] - 2025-12-27

### Added
- Support for .mmd output format using the `--output-format` flag.
- Support for single input file stdin using `-` as the path.
- Support for single output file using the `--output` flag and `-` as the path.
- Composition detection now supports union/tuple annotations (e.g. `Engine | Wheel`, `Union[Engine, Wheel]`) and will emit composition edges for each candidate type.

### Changed
- Dependency + CI maintenance (migrated from Dependabot to Renovate; routine updates across Rust crates, GitHub Actions, and the web app toolchain).

## [0.1.5] - 2025-11-24

### Added
- Web app for interactive editing/viewing.
- Web development improvements (base path handling for deployment/WASM loading, editor UX refinements, configurable dev port).
- Support for generics.
- Python 3.14 support.
- Improved handling of ABCs, Enums, `@final`, `@staticmethod`, and magic method return type inference.

### Changed
- Internal refactors and output generation improvements (including recursion/output behavior tweaks).
- CI workflow adjustments for version-tag releases.
- Dependency updates (many via automation).

### Fixed
- WASM-related fixes.
- Lint fixes and various diagram/stub correctness fixes.

## [0.1.4] - 2024-09-22

### Added
- Ability to exclude files from parsing.
- Minimal class diagram test coverage additions.

## [0.1.3] - 2024-09-20

### Added
- Mermaid labels and expanded tests.
- `@final` support for methods.

### Changed
- Improved base class resolution and relationship output logic (e.g., skip `object` base class).
- Deduplication of identical overloads.
- Improved output recursion behavior.

### Fixed
- Stub diagram generation fixes.
- Trailing whitespace trimming in class definitions.

## [0.1.2] - 2024-09-17

### Added
- Override decorator detection.
- Runtime tests in CI.

### Changed
- Refactored diagram logic out of the `Mermaider` struct.
- Improved class relationship naming (qualified names) and formatting (backticks).
- Improved output directory handling for single-file mode.

## [0.1.1] - 2024-09-17

### Added
- CI running for all pushes, plus an `all-builds-pass` aggregate job for branch protections.
- Package metadata and README installation improvements.

## [0.1.0] - 2024-09-17

### Added
- Initial release: Mermaid class diagram generation from Python code.
- CLI improvements (titles, output directory options, nested folder structure for `-m`).
- Basic import resolution and checker logic inspired and powered by Ruff.
- Documentation (README badges, known issues, and test suite notes).
