use crate::args::Args;
use crate::settings::FileResolverSettings;
use pymermaider_wasm::class_diagram::ClassDiagram;
use pymermaider_wasm::render::mermaid_renderer::RenderOptions;
use pymermaider_wasm::render::output_format::OutputFormat;

use globset::Candidate;
use ignore::{types::TypesBuilder, WalkBuilder};
use log::{debug, error};
use std::path::{Path, PathBuf};

pub struct Mermaider {
    args: Args,
    file_settings: FileResolverSettings,
}

impl Mermaider {
    pub const fn new(args: Args, file_settings: FileResolverSettings) -> Self {
        Self {
            args,
            file_settings,
        }
    }

    pub const fn args(&self) -> &Args {
        &self.args
    }

    /// Format raw Mermaid output according to the configured `output_format`.
    ///
    /// - `Md`: wraps in a fenced Markdown `mermaid` block and ensures a trailing newline.
    /// - `Mmd`: emits raw Mermaid and ensures a trailing newline.
    pub fn format_output(&self, raw: &str) -> String {
        let raw = raw.trim_end();
        match self.args.output_format {
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
            eprintln!("{} does not exist.", root.display());
            return vec![];
        }

        if root.is_file() {
            let mut diagram = self.make_mermaid(std::slice::from_ref(&root.to_path_buf()));
            if !self.args.no_title {
                diagram.path = root.to_string_lossy().into_owned();
            }
            return vec![diagram];
        }

        if root.is_dir() {
            let parsed_files = self.parse_folder(root);

            if self.args.multiple_files {
                use rayon::prelude::*;
                return parsed_files
                    .par_iter()
                    .map(|parsed_file| {
                        let mut diagram = self.make_mermaid_for_file(parsed_file);
                        if !self.args.no_title {
                            diagram.path = parsed_file
                                .strip_prefix(root)
                                .unwrap_or(parsed_file.as_path())
                                .to_string_lossy()
                                .into_owned();
                        }
                        diagram
                    })
                    .collect();
            }

            let mut diagram = self.make_mermaid(&parsed_files);
            if !self.args.no_title {
                let canonical_path = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
                canonical_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("diagram")
                    .clone_into(&mut diagram.path);
            }

            return vec![diagram];
        }

        vec![]
    }

    fn make_mermaid_for_file(&self, file: &Path) -> ClassDiagram {
        let options = RenderOptions {
            direction: self.args.direction,
            hide_private_members: self.args.hide_private_members,
        };
        let mut diagram = ClassDiagram::new(options);
        if let Ok(source) = std::fs::read_to_string(file) {
            diagram.add_file(&source, file);
        }
        diagram
    }

    fn make_mermaid(&self, parsed_files: &[PathBuf]) -> ClassDiagram {
        use rayon::prelude::*;
        let options = RenderOptions {
            direction: self.args.direction,
            hide_private_members: self.args.hide_private_members,
        };
        let per_file: Vec<ClassDiagram> = parsed_files
            .par_iter()
            .map(|file| self.make_mermaid_for_file(file))
            .collect();
        per_file
            .into_iter()
            .fold(ClassDiagram::new(options), ClassDiagram::merge)
    }

    fn parse_folder(&self, path: &Path) -> Vec<PathBuf> {
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
                    debug!("Parsing path: {}", path.display());

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
                        debug!("Skipping excluded path: {}", path.display());
                        continue;
                    }

                    // When include is set, only keep files matching at least one pattern
                    if let Some(ref include) = self.file_settings.include {
                        if !include.is_match_candidate(&path_candidate) {
                            debug!("Skipping path not matching include: {path:?}");
                            continue;
                        }
                    }

                    parsed_files.push(path.to_path_buf());
                }
                Err(err) => {
                    error!("Error walking path: {err:?}");
                }
            }
        }

        parsed_files.sort_unstable();
        parsed_files
    }
}

#[cfg(test)]
mod tests {
    use pymermaider_wasm::render::renderer::DiagramDirection;
    use ruff_linter::settings::types::{FilePattern, FilePatternSet, GlobPath};
    use std::io::Write as _;
    use std::path::Path;

    use crate::settings::DEFAULT_EXCLUDES;

    use super::*;
    use anyhow::Result;
    use tempfile::{Builder, TempDir};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn default_args() -> Args {
        Args {
            path: String::new(),
            multiple_files: false,
            output_dir: "./output".to_string(),
            output_format: OutputFormat::default(),
            output: None,
            exclude: None,
            extend_exclude: None,
            include: None,
            direction: DiagramDirection::default(),
            no_title: false,
            hide_private_members: false,
        }
    }

    fn default_settings(project_root: &Path) -> FileResolverSettings {
        FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::default(),
            include: None,
            project_root: project_root.to_path_buf(),
        }
    }

    #[test]
    fn format_output_md_wraps() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        let mermaider = Mermaider::new(default_args(), default_settings(temp.path()));

        let md = mermaider.format_output("classDiagram\n  class A\n");
        assert!(md.starts_with("```mermaid\n"));
        assert!(md.contains("classDiagram"));
        assert!(md.ends_with("```\n"));
        Ok(())
    }

    #[test]
    fn format_output_mmd_is_raw() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        let mut args = default_args();
        args.output_format = OutputFormat::Mmd;
        let mermaider = Mermaider::new(args, default_settings(temp.path()));

        let mmd = mermaider.format_output("classDiagram\n  class A\n");
        assert!(!mmd.contains("```mermaid"));
        assert!(mmd.contains("classDiagram"));
        assert!(mmd.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn test_single_file() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        std::fs::File::create(temp.path().join("test.py"))?.write_all(b"class Test: ...")?;

        let mermaider = Mermaider::new(default_args(), default_settings(temp.path()));
        let diagrams = mermaider.generate_diagrams();
        assert_eq!(diagrams.len(), 1);
        Ok(())
    }

    #[test]
    fn test_exclude_files() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        std::fs::File::create(temp.path().join("test.py"))?.write_all(b"class Test: ...")?;

        let excluded_dir = Builder::new()
            .prefix("exclude-me-")
            .tempdir_in(temp.path())?;
        std::fs::File::create(excluded_dir.path().join("exclusion_test.py"))?
            .write_all(b"class ExclusionTest: ...")?;

        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::try_from_iter(vec![FilePattern::User(
                "exclude-me-*".to_owned(),
                GlobPath::normalize("exclude-me-*", &temp),
            )])?,
            include: None,
            project_root: temp.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(default_args(), settings);
        let diagrams = mermaider.generate_diagrams();
        assert_eq!(diagrams.len(), 1);
        let rendered = diagrams[0].render().unwrap();
        assert!(!rendered.contains("ExclusionTest"));
        Ok(())
    }

    #[test]
    fn test_include_files() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;

        let models_dir = temp.path().join("models");
        std::fs::create_dir_all(&models_dir)?;
        std::fs::File::create(models_dir.join("user.py"))?.write_all(b"class User: ...")?;

        let views_dir = temp.path().join("views");
        std::fs::create_dir_all(&views_dir)?;
        std::fs::File::create(views_dir.join("home.py"))?.write_all(b"class HomeView: ...")?;

        // Include only models directory
        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::default(),
            include: Some(
                FilePatternSet::try_from_iter(vec![FilePattern::User(
                    "**/models/*".to_owned(),
                    GlobPath::normalize("**/models/*", &temp),
                )])
                .unwrap(),
            ),
            project_root: temp.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(default_args(), settings);
        let diagrams = mermaider.generate_diagrams();

        // Should have one combined diagram with only User (from models)
        assert_eq!(diagrams.len(), 1);
        let rendered = diagrams[0].render().unwrap();
        assert!(rendered.contains("class User"));
        assert!(!rendered.contains("class HomeView"));
        Ok(())
    }

    #[test]
    fn test_include_and_exclude_same_file_exclude_wins() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;

        let models_dir = temp.path().join("models");
        std::fs::create_dir_all(&models_dir)?;
        std::fs::File::create(models_dir.join("user.py"))?.write_all(b"class User: ...")?;
        std::fs::File::create(models_dir.join("settings.py"))?.write_all(b"class Settings: ...")?;

        // Include models, but exclude user.py. Both include and exclude match user.py - exclude should win.
        let settings = FileResolverSettings {
            exclude: FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec()).unwrap(),
            extend_exclude: FilePatternSet::try_from_iter(vec![FilePattern::User(
                "**/models/user.py".to_owned(),
                GlobPath::normalize("**/models/user.py", &temp),
            )])?,
            include: Some(
                FilePatternSet::try_from_iter(vec![FilePattern::User(
                    "**/models/*".to_owned(),
                    GlobPath::normalize("**/models/*", &temp),
                )])
                .unwrap(),
            ),
            project_root: temp.path().to_path_buf(),
        };

        let mermaider = Mermaider::new(default_args(), settings);
        let diagrams = mermaider.generate_diagrams();

        assert_eq!(diagrams.len(), 1);
        let rendered = diagrams[0].render().unwrap();
        assert!(
            !rendered.contains("class User"),
            "exclude should win over include"
        );
        assert!(rendered.contains("class Settings"));
        Ok(())
    }

    #[test]
    fn test_no_title_omits_path() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        std::fs::File::create(temp.path().join("test.py"))?.write_all(b"class Test: ...")?;

        let mut args = default_args();
        args.no_title = true;
        let mermaider = Mermaider::new(args, default_settings(temp.path()));
        let diagrams = mermaider.generate_diagrams();

        assert_eq!(diagrams.len(), 1);
        assert!(diagrams[0].path.is_empty());

        let rendered = diagrams[0].render().unwrap();
        assert!(!rendered.contains("title:"));
        Ok(())
    }

    #[test]
    fn test_parallel_processing_is_deterministic() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        std::fs::File::create(temp.path().join("base.py"))?.write_all(b"class Base: ...")?;
        for i in 1..=5 {
            let content = format!("class Child{i}(Base):\n    x: Base\n");
            std::fs::File::create(temp.path().join(format!("child_{i}.py")))?
                .write_all(content.as_bytes())?;
        }

        let render_with_threads = |num_threads: usize| -> String {
            let mermaider = Mermaider::new(default_args(), default_settings(temp.path()));
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap();
            pool.install(|| {
                let diagrams = mermaider.generate_diagrams();
                assert_eq!(diagrams.len(), 1);
                diagrams[0].render().unwrap_or_default()
            })
        };

        let single_threaded = render_with_threads(1);
        let multi_threaded = render_with_threads(4);

        assert_eq!(single_threaded, multi_threaded);
        Ok(())
    }

    #[test]
    fn test_parallel_multiple_files_parity() -> Result<()> {
        init_logger();
        let temp = TempDir::new()?;
        std::fs::File::create(temp.path().join("base.py"))?.write_all(b"class Base: ...")?;
        for i in 1..=5 {
            let content = format!("class Child{i}(Base):\n    x: Base\n");
            std::fs::File::create(temp.path().join(format!("child_{i}.py")))?
                .write_all(content.as_bytes())?;
        }

        let num_py_files = 6usize;

        let run_with_threads = |num_threads: usize| -> Vec<(String, Option<String>)> {
            let mut args = default_args();
            args.multiple_files = true;
            let mermaider = Mermaider::new(args, default_settings(temp.path()));
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap();
            pool.install(|| {
                let diagrams = mermaider.generate_diagrams();
                assert_eq!(diagrams.len(), num_py_files);
                let mut pairs: Vec<(String, Option<String>)> = diagrams
                    .iter()
                    .map(|d| (d.path.clone(), d.render()))
                    .collect();
                pairs.sort_by(|a, b| a.0.cmp(&b.0));
                pairs
            })
        };

        let single_threaded = run_with_threads(1);
        let multi_threaded = run_with_threads(4);

        assert_eq!(single_threaded, multi_threaded);
        Ok(())
    }
}
