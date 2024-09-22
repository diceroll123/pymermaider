use crate::{class_diagram::ClassDiagram, settings::FileResolverSettings};

use globset::Candidate;
use ignore::{types::TypesBuilder, WalkBuilder};
use std::path::{Path, PathBuf};

pub struct Mermaider {
    file_settings: FileResolverSettings,
    multiple_files: bool,
    files_written: usize,
}
impl Mermaider {
    pub fn new(file_settings: FileResolverSettings, multiple_files: bool) -> Self {
        Self {
            file_settings,
            multiple_files,
            files_written: 0,
        }
    }

    /// Get the amount of files written.
    pub fn get_written_files(&self) -> usize {
        self.files_written
    }

    pub fn process(&mut self) {
        let path = &self.file_settings.project_root;
        let output_directory = &self.file_settings.output_directory;

        if !path.exists() {
            println!("{:?} does not exist.", path);
            return;
        }

        if path.is_file() {
            let mut diagram = self.make_mermaid(vec![path.to_path_buf()]);
            diagram.path = path.to_string_lossy().to_string();

            let wrote_file = diagram.write_to_file(output_directory);
            if wrote_file {
                self.files_written += 1;
            }
        } else if path.is_dir() {
            let parsed_files = self.parse_folder(path).unwrap();

            if self.multiple_files {
                for parsed_file in parsed_files.iter() {
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

        for file in parsed_files.iter() {
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
                    debug!("Parsing path: {:?}", path);

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
                        debug!("Skipping excluded path: {:?}", path);
                        continue;
                    }

                    parsed_files.push(path.to_path_buf());
                }
                Err(err) => {
                    error!("Error walking path: {:?}", err);
                }
            }
        }

        Ok(parsed_files)
    }
}
