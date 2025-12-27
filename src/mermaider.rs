use crate::output_format::OutputFormat;
use crate::settings::FileResolverSettings;
use pymermaider_lib::class_diagram::ClassDiagram;
use pymermaider_lib::renderer::DiagramDirection;

use globset::Candidate;
use ignore::{types::TypesBuilder, WalkBuilder};
use log::{debug, error};
use std::path::{Path, PathBuf};

pub struct Mermaider {
    file_settings: FileResolverSettings,
    multiple_files: bool,
    output_format: OutputFormat,
    direction: DiagramDirection,
}
impl Mermaider {
    pub const fn new(
        file_settings: FileResolverSettings,
        multiple_files: bool,
        output_format: OutputFormat,
        direction: DiagramDirection,
    ) -> Self {
        Self {
            file_settings,
            multiple_files,
            output_format,
            direction,
        }
    }

    /// Format raw Mermaid output according to the configured `output_format`.
    ///
    /// - `Md`: wraps in a fenced ```mermaid Markdown block and ensures a trailing newline.
    /// - `Mmd`: emits raw Mermaid and ensures a trailing newline.
    pub fn format_output(&self, raw: &str) -> String {
        let raw = raw.trim_end();
        match self.output_format {
            OutputFormat::Md => format!("```mermaid\n{raw}\n```\n"),
            OutputFormat::Mmd => format!("{raw}\n"),
        }
    }

    /// Generate one or more class diagrams without writing them to disk.
    ///
    /// - If the project root is a file, returns one diagram.
    /// - If the project root is a directory:
    ///   - with `multiple_files=true`, returns one diagram per Python file.
    ///   - otherwise returns one combined diagram.
    pub fn generate_diagrams(&self) -> Vec<ClassDiagram> {
        let root = self.file_settings.project_root.as_path();

        if !root.exists() {
            eprintln!("{root:?} does not exist.");
            return vec![];
        }

        if root.is_file() {
            let mut diagram = self.make_mermaid(vec![root.to_path_buf()]);
            diagram.path = root.to_string_lossy().to_string();
            return vec![diagram];
        }

        if root.is_dir() {
            let parsed_files = match self.parse_folder(root) {
                Ok(files) => files,
                Err(_) => return vec![],
            };

            if self.multiple_files {
                let mut diagrams = Vec::with_capacity(parsed_files.len());
                for parsed_file in &parsed_files {
                    let mut diagram = self.make_mermaid(vec![parsed_file.clone()]);
                    diagram.path = parsed_file
                        .strip_prefix(root)
                        .unwrap_or(parsed_file.as_path())
                        .to_string_lossy()
                        .to_string();
                    diagrams.push(diagram);
                }
                return diagrams;
            }

            let canonical_path = match root.canonicalize() {
                Ok(p) => p,
                Err(_) => root.to_path_buf(),
            };

            let mut diagram = self.make_mermaid(parsed_files);
            diagram.path = canonical_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("diagram")
                .to_owned();

            return vec![diagram];
        }

        vec![]
    }

    fn make_mermaid(&self, parsed_files: Vec<PathBuf>) -> ClassDiagram {
        let mut class_diagram = ClassDiagram::new(self.direction);

        for file in &parsed_files {
            let source = match std::fs::read_to_string(file) {
                Ok(content) => content,
                Err(_) => continue,
            };

            class_diagram.add_file(source, file);
        }

        class_diagram
    }

    fn parse_folder(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut parsed_files = vec![];

        let types = TypesBuilder::new()
            .add_defaults()
            .select("python")
            .build()
            .expect("Failed to build Python types");

        for result in WalkBuilder::new(path).types(types).build() {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    debug!("Parsing path: {path:?}");

                    let path_candidate = Candidate::new(path);

                    if path.is_dir() {
                        // we're only doing files here
                        continue;
                    }

                    if self
                        .file_settings
                        .exclude
                        .is_match_candidate(&path_candidate)
                        || self
                            .file_settings
                            .extend_exclude
                            .is_match_candidate(&path_candidate)
                    {
                        debug!("Skipping excluded path: {path:?}");
                        continue;
                    }

                    parsed_files.push(path.to_path_buf());
                }
                Err(err) => {
                    error!("Error walking path: {err:?}");
                }
            }
        }

        parsed_files.sort();
        Ok(parsed_files)
    }
}

#[cfg(test)]
mod tests {
    use ruff_linter::settings::types::GlobPath;
    use std::io::Write as _;

    use crate::settings::DEFAULT_EXCLUDES;

    use super::*;
    use anyhow::Result;
    use ruff_linter::settings::types::{FilePattern, FilePatternSet};
    use tempfile::{Builder, TempDir};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn format_output_md_wraps() -> Result<()> {
        init_logger();
        let temp_project_dir = TempDir::new()?;

        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::default(),
            project_root: temp_project_dir.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(
            settings,
            true,
            OutputFormat::Md,
            DiagramDirection::default(),
        );
        let md = mermaider.format_output("classDiagram\n  class A\n");
        assert!(md.starts_with("```mermaid\n"));
        assert!(md.contains("classDiagram"));
        assert!(md.ends_with("```\n"));
        Ok(())
    }

    #[test]
    fn format_output_mmd_is_raw() -> Result<()> {
        init_logger();
        let temp_project_dir = TempDir::new()?;

        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::default(),
            project_root: temp_project_dir.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(
            settings,
            true,
            OutputFormat::Mmd,
            DiagramDirection::default(),
        );
        let mmd = mermaider.format_output("classDiagram\n  class A\n");
        assert!(!mmd.contains("```mermaid"));
        assert!(mmd.contains("classDiagram"));
        assert!(mmd.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_single_file() -> Result<()> {
        init_logger();
        let temp_project_dir = TempDir::new()?;
        let mut test_file = std::fs::File::create(temp_project_dir.path().join("test.py"))?;
        test_file.write_all(b"class Test: ...")?;

        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::default(),
            project_root: temp_project_dir.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(
            settings,
            true,
            OutputFormat::Md,
            DiagramDirection::default(),
        );
        let diagrams = mermaider.generate_diagrams();
        assert_eq!(diagrams.len(), 1);

        Ok(())
    }

    #[test]
    fn test_exclude_files() -> Result<()> {
        init_logger();
        let temp_project_dir = TempDir::new()?;
        let mut test_file = std::fs::File::create(temp_project_dir.path().join("test.py"))?;
        test_file.write_all(b"class Test: ...")?;

        let temp_excluded_dir = Builder::new()
            .prefix("exclude-me-")
            .tempdir_in(temp_project_dir.path())?;

        let mut excluded_file =
            std::fs::File::create(temp_excluded_dir.path().join("exclusion_test.py"))?;
        excluded_file.write_all(b"class ExclusionTest: ...")?;

        let absolute = GlobPath::normalize("exclude-me-*", &temp_project_dir);

        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::try_from_iter(vec![FilePattern::User(
                "exclude-me-*".to_owned(),
                absolute,
            )])?,
            project_root: temp_project_dir.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(
            settings,
            true,
            OutputFormat::Md,
            DiagramDirection::default(),
        );
        let diagrams = mermaider.generate_diagrams();
        assert_eq!(diagrams.len(), 1);

        Ok(())
    }
}
