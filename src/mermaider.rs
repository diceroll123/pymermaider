use crate::{class_diagram::ClassDiagram, settings::FileResolverSettings};

use globset::Candidate;
use ignore::{types::TypesBuilder, WalkBuilder};
use log::{debug, error};
use std::path::{Path, PathBuf};

pub struct Mermaider {
    file_settings: FileResolverSettings,
    multiple_files: bool,
    files_written: usize,
}
impl Mermaider {
    pub const fn new(file_settings: FileResolverSettings, multiple_files: bool) -> Self {
        Self {
            file_settings,
            multiple_files,
            files_written: 0,
        }
    }

    /// Get the amount of files written.
    pub const fn get_written_files(&self) -> usize {
        self.files_written
    }

    pub fn process(&mut self) {
        let path = &self.file_settings.project_root;
        let output_directory = &self.file_settings.output_directory;

        if !path.exists() {
            println!("{path:?} does not exist.");
            return;
        }

        if path.is_file() {
            let mut diagram = self.make_mermaid(vec![path.clone()]);
            diagram.path = path.to_string_lossy().to_string();

            let wrote_file = diagram.write_to_file(output_directory);
            if wrote_file {
                self.files_written += 1;
            }
        } else if path.is_dir() {
            let parsed_files = self.parse_folder(path).unwrap();

            if self.multiple_files {
                for parsed_file in &parsed_files {
                    let mut diagram = self.make_mermaid(vec![parsed_file.clone()]);

                    let diagram_path = Path::new(parsed_file).strip_prefix(path).unwrap();

                    diagram.path = diagram_path.to_string_lossy().to_string();

                    let wrote_file = diagram.write_to_file(output_directory);
                    if wrote_file {
                        self.files_written += 1;
                    }
                }
            } else {
                let canonical_path = path.canonicalize().unwrap();

                let mut diagram = self.make_mermaid(parsed_files);
                diagram.path = canonical_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();

                let wrote_file = diagram.write_to_file(output_directory);
                if wrote_file {
                    self.files_written += 1;
                }
            }
        }
    }

    fn make_mermaid(&self, parsed_files: Vec<PathBuf>) -> ClassDiagram {
        let mut class_diagram = ClassDiagram::new();

        for file in &parsed_files {
            let source = match std::fs::read_to_string(file) {
                Ok(content) => content,
                Err(_) => continue,
            };

            class_diagram.add_to_diagram(source, file);
        }

        class_diagram
    }

    fn parse_folder(&self, path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
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
    fn test_single_file() -> Result<()> {
        init_logger();
        let temp_project_dir = TempDir::new()?;
        let mut test_file = std::fs::File::create(temp_project_dir.path().join("test.py"))?;
        test_file.write_all(b"class Test: ...")?;

        let temp_output_dir = TempDir::new()?;

        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::default(),
            project_root: temp_project_dir.path().to_path_buf(),
            output_directory: temp_output_dir.path().to_path_buf(),
        };

        let mut mermaider = Mermaider::new(settings, true);

        mermaider.process();

        assert_eq!(mermaider.get_written_files(), 1);

        Ok(())
    }

    #[test]
    fn test_exclude_files() -> Result<()> {
        init_logger();
        let temp_project_dir = TempDir::new()?;
        let mut test_file = std::fs::File::create(temp_project_dir.path().join("test.py"))?;
        test_file.write_all(b"class Test: ...")?;

        let temp_output_dir = TempDir::new()?;

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
            output_directory: temp_output_dir.path().to_path_buf(),
        };

        let mut mermaider = Mermaider::new(settings, true);

        mermaider.process();

        assert_eq!(mermaider.get_written_files(), 1);

        Ok(())
    }
}
