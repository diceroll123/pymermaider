use ruff_linter::settings::types::FilePattern;
use ruff_linter::settings::types::GlobPath;
use std::path::Path;
use std::path::PathBuf;

use ruff_linter::settings::types::FilePatternSet;

pub struct FileResolverSettings {
    pub exclude: FilePatternSet,
    pub extend_exclude: FilePatternSet,
    pub include: Option<FilePatternSet>,
    pub project_root: PathBuf,
}

pub static DEFAULT_EXCLUDES: &[FilePattern] = &[
    FilePattern::Builtin(".bzr"),
    FilePattern::Builtin(".direnv"),
    FilePattern::Builtin(".eggs"),
    FilePattern::Builtin(".git"),
    FilePattern::Builtin(".git-rewrite"),
    FilePattern::Builtin(".hg"),
    FilePattern::Builtin(".ipynb_checkpoints"),
    FilePattern::Builtin(".mypy_cache"),
    FilePattern::Builtin(".nox"),
    FilePattern::Builtin(".pants.d"),
    FilePattern::Builtin(".pyenv"),
    FilePattern::Builtin(".pytest_cache"),
    FilePattern::Builtin(".pytype"),
    FilePattern::Builtin(".ruff_cache"),
    FilePattern::Builtin(".svn"),
    FilePattern::Builtin(".tox"),
    FilePattern::Builtin(".venv"),
    FilePattern::Builtin(".vscode"),
    FilePattern::Builtin("__pypackages__"),
    FilePattern::Builtin("_build"),
    FilePattern::Builtin("buck-out"),
    FilePattern::Builtin("dist"),
    FilePattern::Builtin("node_modules"),
    FilePattern::Builtin("site-packages"),
    FilePattern::Builtin("venv"),
];

/// Build a `FilePatternSet` from a list of user-supplied glob patterns, normalizing each
/// pattern against `project_root`. Returns an error message (rather than panicking) if any
/// pattern fails to parse.
fn build_pattern_set(
    patterns: Option<Vec<String>>,
    project_root: &Path,
) -> Result<FilePatternSet, String> {
    let entries = patterns.unwrap_or_default().into_iter().map(|pattern| {
        let absolute = GlobPath::normalize(&pattern, project_root);
        FilePattern::User(pattern, absolute)
    });
    FilePatternSet::try_from_iter(entries).map_err(|e| e.to_string())
}

impl FileResolverSettings {
    /// Construct a `FileResolverSettings` from the raw CLI-supplied pattern lists.
    ///
    /// Returns `Err` (instead of panicking) if any exclude/extend-exclude/include pattern
    /// fails to parse as a valid glob.
    pub fn new(
        exclude_patterns: Option<Vec<String>>,
        extend_exclude_patterns: Option<Vec<String>>,
        include_patterns: Option<Vec<String>>,
        project_root: PathBuf,
    ) -> Result<Self, String> {
        let exclude = match exclude_patterns {
            Some(patterns) => build_pattern_set(Some(patterns), &project_root)?,
            None => FilePatternSet::try_from_iter(DEFAULT_EXCLUDES.to_vec())
                .map_err(|e| e.to_string())?,
        };
        let extend_exclude = build_pattern_set(extend_exclude_patterns, &project_root)?;
        let include = include_patterns
            .map(|patterns| build_pattern_set(Some(patterns), &project_root))
            .transpose()?;

        Ok(Self {
            exclude,
            extend_exclude,
            include,
            project_root,
        })
    }
}
