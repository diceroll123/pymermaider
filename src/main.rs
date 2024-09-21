mod checker;
mod class_diagram;
mod mermaider;
mod parameter_generator;
mod settings;

extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::path::PathBuf;

use clap::Parser;
use mermaider::Mermaider;
use ruff_linter::{
    fs,
    settings::types::{FilePattern, FilePatternSet},
};
use ruff_python_ast::{self as ast};
use settings::{FileResolverSettings, DEFAULT_EXCLUDES};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to a file or directory
    #[arg()]
    path: String,

    #[arg(
        short,
        long,
        default_value = "false",
        long_help = "Process each file individually, outputting a mermaid file for each file. Only used when path is a directory."
    )]
    multiple_files: bool,

    /// Output directory for mermaid files.
    #[arg(short, long, default_value = "./output")]
    output_dir: String,

    /// List of paths, used to omit files and/or directories from analysis.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "FILE_PATTERN",
        help_heading = "File selection"
    )]
    pub exclude: Option<Vec<String>>,
    /// Like --exclude, but adds additional files and directories on top of those already excluded.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "FILE_PATTERN",
        help_heading = "File selection"
    )]
    pub extend_exclude: Option<Vec<String>>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let project_root = PathBuf::from(args.path);

    let file_settings = FileResolverSettings {
        project_root: project_root.clone(),
        output_directory: args.output_dir.into(),
        exclude: FilePatternSet::try_from_iter(
            args.exclude
                .map(|paths| {
                    paths
                        .into_iter()
                        .map(|pattern| {
                            let absolute = fs::normalize_path_to(&pattern, &project_root);
                            FilePattern::User(pattern, absolute)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or(DEFAULT_EXCLUDES.to_vec()),
        )
        .unwrap(),
        extend_exclude: FilePatternSet::try_from_iter(
            args.extend_exclude
                .map(|paths| {
                    paths
                        .into_iter()
                        .map(|pattern| {
                            let absolute = fs::normalize_path_to(&pattern, &project_root);
                            FilePattern::User(pattern, absolute)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        )
        .unwrap(),
    };

    let mut mermaider = Mermaider::new(file_settings, args.multiple_files);

    mermaider.process();

    let written = mermaider.get_written_files();

    println!("Files written: {}", written);
}
