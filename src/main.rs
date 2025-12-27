mod args;
mod mermaider;
mod output_format;
mod settings;

use std::path::PathBuf;

use args::Args;
use clap::Parser;
use log::info;
use mermaider::Mermaider;
use pymermaider_lib::class_diagram;
use ruff_linter::settings::types::{FilePattern, FilePatternSet, GlobPath};
use settings::{FileResolverSettings, DEFAULT_EXCLUDES};
use std::io::Read as _;
use std::io::Write as _;

fn main() {
    env_logger::init();

    let args = Args::parse();

    // Special-case stdin input: `pymermaider -` reads Python source from stdin.
    // For path normalization in exclude patterns, use the current working directory.
    let is_stdin = args.path == "-";
    let project_root = if is_stdin {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        PathBuf::from(&args.path)
    };
    let output_dir_path = PathBuf::from(&args.output_dir);

    let file_settings = FileResolverSettings {
        project_root: project_root.clone(),
        exclude: FilePatternSet::try_from_iter(args.exclude.map_or(
            DEFAULT_EXCLUDES.to_vec(),
            |paths| {
                paths
                    .into_iter()
                    .map(|pattern| {
                        let absolute = GlobPath::normalize(&pattern, &project_root);
                        FilePattern::User(pattern, absolute)
                    })
                    .collect::<Vec<_>>()
            },
        ))
        .unwrap(),
        extend_exclude: FilePatternSet::try_from_iter(
            args.extend_exclude
                .map(|paths| {
                    paths
                        .into_iter()
                        .map(|pattern| {
                            let absolute = GlobPath::normalize(&pattern, &project_root);
                            FilePattern::User(pattern, absolute)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        )
        .unwrap(),
    };

    let mermaider = Mermaider::new(
        file_settings,
        args.multiple_files,
        args.output_format,
        args.direction,
    );

    let diagrams = if is_stdin {
        if args.multiple_files {
            eprintln!("--multiple-files is not compatible with stdin input (PATH='-').");
            std::process::exit(2);
        }

        let mut source = String::new();
        std::io::stdin()
            .read_to_string(&mut source)
            .unwrap_or_else(|e| panic!("Failed to read stdin: {e}"));

        let mut diagram = class_diagram::ClassDiagram::new(args.direction);
        diagram.add_source(source);

        vec![diagram]
    } else {
        mermaider.generate_diagrams()
    };

    // If --output is provided, render a single diagram to stdout or a specific file.
    // This is only valid when we're generating a single diagram (i.e. not multiple per-file outputs).
    if let Some(output) = args.output {
        if diagrams.len() > 1 {
            eprintln!("--output is not compatible with --multiple-files (it would produce multiple outputs). Use --output-dir instead.");
            std::process::exit(2);
        }

        let raw = diagrams
            .first()
            .and_then(|d| d.render())
            .unwrap_or_default();
        let content = mermaider.format_output(&raw);

        if output == "-" {
            std::io::stdout()
                .write_all(content.as_bytes())
                .unwrap_or_else(|e| panic!("Failed to write stdout: {e}"));
        } else {
            std::fs::write(&output, content)
                .unwrap_or_else(|e| panic!("Failed to write file {output:?}: {e}"));
        }
    } else {
        let extension = args.output_format.extension();
        let output_dir = output_dir_path;

        let mut written = 0usize;
        for diagram in &diagrams {
            if diagram.is_empty() {
                info!("No classes found for {0:?}.", diagram.path);
                continue;
            }

            let path = format!(
                "{0}/{1}.{2}",
                output_dir.to_string_lossy(),
                diagram.path,
                extension
            );

            if let Some(parent_dir) = std::path::Path::new(&path).parent() {
                std::fs::create_dir_all(parent_dir)
                    .unwrap_or_else(|e| panic!("Failed to create directory {parent_dir:?}: {e}"));
            }

            let raw = diagram.render().unwrap_or_default();
            let content = mermaider.format_output(&raw);
            std::fs::write(&path, content)
                .unwrap_or_else(|e| panic!("Failed to write file {path:?}: {e}"));
            eprintln!("Mermaid file written to: {path:?}");
            written += 1;
        }

        eprintln!("Files written: {written}");
    }
}
