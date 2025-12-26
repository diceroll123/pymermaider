# Changelog

## [Unreleased]

### Added
- Support for .mmd output format using the `--output-format` flag.
- Support for single input file stdin using `-` as the path.
- Support for single output file using the `--output` flag and `-` as the path.

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
