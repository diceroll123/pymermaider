use ruff_linter::settings::types::FilePattern;
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
