mod checker;
mod class_diagram;
mod mermaider;
mod parameter_generator;

extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use clap::Parser;
use mermaider::Mermaider;
use ruff_python_ast::{self as ast};

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
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let mut mermaider = Mermaider::new(args.path, args.output_dir, args.multiple_files);

    mermaider.process();

    let written = mermaider.get_written_files();

    println!("Files written: {}", written);
}
