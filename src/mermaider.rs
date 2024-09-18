use crate::checker;
use crate::class_diagram::ClassDiagram;
use checker::Checker;
use ignore::{types::TypesBuilder, WalkBuilder};
use ruff_linter::source_kind::SourceKind;
use ruff_python_ast::{self as ast, PySourceType};
use ruff_python_codegen::Stylist;
use ruff_python_parser::parse_unchecked_source;
use ruff_python_semantic::{Module, ModuleKind, ModuleSource, SemanticModel};
use ruff_source_file::Locator;
use std::path::Path;

pub struct Mermaider {
    path: String,
    output_directory: String,
    multiple_files: bool,
    files_written: usize,
}
impl Mermaider {
    pub fn new(path: String, output_directory: String, multiple_files: bool) -> Self {
        Self {
            path,
            output_directory,
            multiple_files,
            files_written: 0,
        }
    }

    /// Get the amount of files written.
    pub fn get_written_files(&self) -> usize {
        self.files_written
    }

    pub fn process(&mut self) {
        let path = Path::new(&self.path);

        if !path.exists() {
            println!("{:?} does not exist.", path);
            return;
        }

        if path.is_file() {
            let mut diagram = self.make_mermaid(vec![path.to_str().unwrap().to_string()]);
            diagram.path = path.file_name().unwrap().to_str().unwrap().to_owned();

            let wrote_file = diagram.write_to_file(&self.output_directory);
            if wrote_file {
                self.files_written += 1;
            }
        } else if path.is_dir() {
            let parsed_files = self.parse_folder(path).unwrap();

            if self.multiple_files {
                for parsed_file in parsed_files.iter() {
                    let path_folder_name = Path::new(parsed_file)
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let title = Path::new(parsed_file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let mut diagram = self.make_mermaid(vec![parsed_file.clone()]);
                    diagram.path = format!("{path_folder_name}/{title}");

                    let wrote_file = diagram.write_to_file(&self.output_directory);
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

                let wrote_file = diagram.write_to_file(&self.output_directory);
                if wrote_file {
                    self.files_written += 1;
                }
            }
        }
    }

    fn make_mermaid(&self, parsed_files: Vec<String>) -> ClassDiagram {
        let mut class_diagram = ClassDiagram::new();

        for file in parsed_files.iter() {
            let source_type = PySourceType::from(file);

            let source = match std::fs::read_to_string(file) {
                Ok(content) => content,
                Err(_) => continue,
            };

            let source_kind = SourceKind::Python(source);

            let locator = Locator::new(source_kind.source_code());

            let parsed = parse_unchecked_source(source_kind.source_code(), source_type);

            let stylist = Stylist::from_tokens(parsed.tokens(), &locator);

            let python_ast = parsed.into_suite();
            let module = Module {
                kind: ModuleKind::Module,
                source: ModuleSource::File(Path::new(file)),
                python_ast: &python_ast,
                name: None,
            };
            let semantic = SemanticModel::new(&[], Path::new(file), module);
            let mut checker = Checker::new(&stylist, semantic);
            checker.see_imports(&python_ast);

            for stmt in python_ast.iter() {
                if let ast::Stmt::ClassDef(class) = stmt {
                    // we only care about class definitions
                    class_diagram.add_class(&checker, class, 1);
                }
            }
        }

        class_diagram
    }

    fn parse_folder(&self, path: &Path) -> std::io::Result<Vec<String>> {
        let mut parsed_files = vec![];

        let types = TypesBuilder::new()
            .add_defaults()
            .select("python")
            .build()
            .unwrap();

        for result in WalkBuilder::new(path).types(types).build() {
            match result {
                Ok(entry) => {
                    if entry.path().is_dir() {
                        // we're only doing files here
                        continue;
                    }

                    if let Some(filename) = entry.path().to_str() {
                        parsed_files.push(filename.to_string());
                    }
                }
                Err(err) => {
                    error!("Error walking path: {:?}", err);
                }
            }
        }

        Ok(parsed_files)
    }
}
