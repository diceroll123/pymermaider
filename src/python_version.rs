use log::debug;
use ruff_python_ast::PythonVersion;
use ruff_workspace::pyproject::find_fallback_target_version;
use std::path::Path;

/// Detect the target Python version for the project containing `start`.
///
/// Uses ruff's own inference: walks up from `start` (a file or directory) to the nearest
/// `pyproject.toml` with `project.requires-python` and takes the lowest supported version.
pub fn detect(start: &Path) -> Option<PythonVersion> {
    let start = std::path::absolute(start).unwrap_or_else(|_| start.to_path_buf());
    let version: PythonVersion = find_fallback_target_version(&start)?.into();
    debug!(
        "Detected Python {version} from requires-python near {}",
        start.display()
    );
    Some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PY310: PythonVersion = PythonVersion::PY310;

    #[test]
    fn detects_walking_up_from_file() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("pyproject.toml"),
            "[project]\nname = \"x\"\nrequires-python = \">=3.10\"\n",
        )
        .unwrap();
        let sub = temp.path().join("pkg");
        std::fs::create_dir(&sub).unwrap();
        let file = sub.join("a.py");
        std::fs::write(&file, "").unwrap();
        assert_eq!(detect(&file), Some(PY310));
        assert_eq!(detect(&sub), Some(PY310));
    }
}
