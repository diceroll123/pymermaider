mod args;
mod mermaider;
mod settings;

use std::path::PathBuf;

use args::Args;
use clap::Parser;
use log::info;
use mermaider::Mermaider;
use pymermaider_wasm::class_diagram;
use settings::FileResolverSettings;
use std::io::Read as _;
use std::io::Write as _;

fn main() {
    env_logger::init();

    let mut args = Args::parse();

    // Special-case stdin input: `pymermaider -` reads Python source from stdin.
    // For path normalization in exclude patterns, use the current working directory.
    let is_stdin = args.path == "-";
    let project_root = if is_stdin {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        PathBuf::from(&args.path)
    };
    let output_dir_path = PathBuf::from(&args.output_dir);

    // Take ownership of file selection patterns (Mermaider doesn't need them after FileResolverSettings is built)
    let exclude_patterns = args.exclude.take();
    let extend_exclude_patterns = args.extend_exclude.take();
    let include_patterns = args.include.take();

    let file_settings = match FileResolverSettings::new(
        exclude_patterns,
        extend_exclude_patterns,
        include_patterns,
        project_root,
    ) {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("error: invalid file pattern: {e}");
            std::process::exit(2);
        }
    };

    let mermaider = Mermaider::new(args, file_settings);

    let diagrams = if is_stdin {
        if mermaider.args().multiple_files {
            eprintln!("--multiple-files is not compatible with stdin input (PATH='-').");
            std::process::exit(2);
        }

        let mut source = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut source) {
            eprintln!("error: failed to read stdin: {e}");
            std::process::exit(1);
        }

        let options = pymermaider_wasm::render::mermaid_renderer::RenderOptions {
            direction: mermaider.args().direction,
            hide_private_members: mermaider.args().hide_private_members,
        };
        let mut diagram = class_diagram::ClassDiagram::new(options);
        diagram.add_source(&source);

        vec![diagram]
    } else {
        mermaider.generate_diagrams()
    };

    // If --output is provided, render a single diagram to stdout or a specific file.
    // This is only valid when we're generating a single diagram (i.e. not multiple per-file outputs).
    if let Some(ref output) = mermaider.args().output {
        if diagrams.len() > 1 {
            eprintln!("--output is not compatible with --multiple-files (it would produce multiple outputs). Use --output-dir instead.");
            std::process::exit(2);
        }

        let raw = diagrams
            .first()
            .and_then(pymermaider_wasm::class_diagram::ClassDiagram::render)
            .unwrap_or_default();
        let content = mermaider.format_output(&raw);

        if output == "-" {
            if let Err(e) = std::io::stdout().write_all(content.as_bytes()) {
                eprintln!("error: failed to write stdout: {e}");
                std::process::exit(1);
            }
        } else if let Err(e) = std::fs::write(output, content) {
            eprintln!("error: failed to write file {output:?}: {e}");
            std::process::exit(1);
        }
    } else {
        let extension = mermaider.args().output_format.extension();
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
                if let Err(e) = std::fs::create_dir_all(parent_dir) {
                    eprintln!(
                        "error: failed to create directory {}: {e}",
                        parent_dir.display()
                    );
                    std::process::exit(1);
                }
            }

            let raw = diagram.render().unwrap_or_default();
            let content = mermaider.format_output(&raw);
            if let Err(e) = std::fs::write(&path, content) {
                eprintln!("error: failed to write file {path:?}: {e}");
                std::process::exit(1);
            }
            eprintln!("Mermaid file written to: {path:?}");
            written += 1;
        }

        eprintln!("Files written: {written}");
    }
}
