//! Snapshot tests over a corpus of Python files (`tests/corpus/*.py`).
//!
//! These pin the *exact* rendered Mermaid output for representative features so that
//! rendering regressions show up as readable snapshot diffs. Snapshots are taken with the
//! default settings (ty inference enabled), matching what the CLI produces out of the box.
//! To update snapshots after an intentional change,
//! review them with `cargo insta review` (or `INSTA_UPDATE=always cargo test`).

mod inferred {
    use std::path::PathBuf;

    use pymermaider_wasm::class_diagram::ClassDiagram;

    const CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus");

    fn corpus_files() -> Vec<PathBuf> {
        let mut files: Vec<_> = std::fs::read_dir(CORPUS_DIR)
            .expect("corpus directory should exist")
            .map(|entry| entry.expect("corpus entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "py"))
            .collect();
        files.sort_unstable();
        files
    }

    #[test]
    fn corpus_snapshots() {
        let mut any = false;
        for path in corpus_files() {
            let name = path
                .file_stem()
                .expect("corpus files have stems")
                .to_string_lossy()
                .to_string();
            let source = std::fs::read_to_string(&path).expect("corpus file should be readable");

            let mut diagram = ClassDiagram::default();
            diagram.add_source(&source);
            assert!(diagram.syntax_errors().is_empty(), "{name}: {path:?}");

            let rendered = diagram
                .render()
                .unwrap_or_else(|| panic!("{name}: {path:?} should render a non-empty diagram"));
            any = true;

            insta::assert_snapshot!(name, rendered);
        }
        assert!(any, "corpus should not be empty");
    }
}
