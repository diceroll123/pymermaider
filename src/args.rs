use clap::Parser;

use crate::output_format::OutputFormat;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to a file or directory. Use '-' to read Python source from stdin.
    #[arg()]
    pub path: String,

    #[arg(
        short,
        long,
        default_value = "false",
        long_help = "Process each file individually, outputting a mermaid file for each file. Only used when path is a directory."
    )]
    pub multiple_files: bool,

    /// Output directory for mermaid files.
    #[arg(short, long, default_value = "./output")]
    pub output_dir: String,

    /// Output file format.
    ///
    /// - md: writes `*.md` files containing a fenced ```mermaid code block
    /// - mmd: writes `*.mmd` files containing raw Mermaid text (no fences)
    #[arg(long, value_enum, default_value_t = OutputFormat::Md)]
    pub output_format: OutputFormat,

    /// Output file path. Use '-' to write to stdout.
    ///
    /// If omitted, output is written to files under --output-dir (the default behavior).
    #[arg(long)]
    pub output: Option<String>,

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
