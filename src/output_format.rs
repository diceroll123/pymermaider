#[cfg(feature = "cli")]
use clap::ValueEnum;

#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Markdown file containing a fenced Mermaid block (three backticks + `mermaid`)
    #[default]
    Md,
    /// Raw Mermaid diagram file (no Markdown fences), suitable for `.mmd`
    Mmd,
}

impl OutputFormat {
    #[cfg(feature = "cli")]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Md => "md",
            Self::Mmd => "mmd",
        }
    }
}
