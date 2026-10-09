use std::process::Command;
use std::process::Stdio;

#[test]
fn file_to_stdout_works() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    let file_path = dir.path().join("a.py");
    std::fs::write(&file_path, "class A: ...\n").expect("write a.py");

    let output = Command::new(exe)
        .arg(file_path.to_string_lossy().to_string())
        .arg("--output")
        .arg("-")
        .output()
        .expect("run pymermaider");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("```mermaid"));
    assert!(stdout.contains("classDiagram"));
    assert!(stdout.contains("A"));
}

#[test]
fn stdin_to_stdout_works() {
    use std::io::Write as _;

    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let mut child = Command::new(exe)
        .arg("-")
        .arg("--output")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn pymermaider");

    {
        let mut stdin = child.stdin.take().expect("stdin piped");
        stdin.write_all(b"class A: ...\n").expect("write to stdin");
    }

    let output = child.wait_with_output().expect("wait pymermaider");
    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("```mermaid"));
    assert!(stdout.contains("classDiagram"));
    assert!(stdout.contains("A"));
}

#[test]
fn file_to_stdout_mmd_works() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    let file_path = dir.path().join("a.py");
    std::fs::write(&file_path, "class A: ...\n").expect("write a.py");

    let output = Command::new(exe)
        .arg(file_path.to_string_lossy().to_string())
        .arg("--output-format")
        .arg("mmd")
        .arg("--output")
        .arg("-")
        .output()
        .expect("run pymermaider");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("```mermaid"));
    assert!(stdout.contains("classDiagram"));
    assert!(stdout.contains("A"));
}

fn collect_paths_with_extension(
    dir: &std::path::Path,
    ext: &str,
    out: &mut Vec<std::path::PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_paths_with_extension(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

#[test]
fn output_dir_mmd_writes_raw_file() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let proj_dir = tempfile::TempDir::new().expect("temp project dir");
    std::fs::write(proj_dir.path().join("a.py"), "class A: ...\n").expect("write a.py");

    let out_dir = tempfile::TempDir::new().expect("temp output dir");

    let output = Command::new(exe)
        .arg(proj_dir.path().to_string_lossy().to_string())
        .arg("--output-dir")
        .arg(out_dir.path().to_string_lossy().to_string())
        .arg("--output-format")
        .arg("mmd")
        .output()
        .expect("run pymermaider");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut mmd_files = Vec::new();
    collect_paths_with_extension(out_dir.path(), "mmd", &mut mmd_files);
    assert_eq!(
        mmd_files.len(),
        1,
        "expected 1 .mmd file, found {mmd_files:?}"
    );

    let contents = std::fs::read_to_string(&mmd_files[0]).expect("read mmd");
    assert!(!contents.contains("```mermaid"));
    assert!(contents.contains("classDiagram"));
    assert!(contents.contains("A"));
}

#[test]
fn include_flag_filters_files() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("models")).expect("create models");
    std::fs::create_dir_all(dir.path().join("views")).expect("create views");
    std::fs::write(
        dir.path().join("models").join("user.py"),
        "class User: ...\n",
    )
    .expect("write user.py");
    std::fs::write(
        dir.path().join("views").join("home.py"),
        "class HomeView: ...\n",
    )
    .expect("write home.py");

    let output = Command::new(exe)
        .arg(dir.path().to_string_lossy().to_string())
        .arg("--include")
        .arg("**/models/*")
        .arg("--output")
        .arg("-")
        .output()
        .expect("run pymermaider");

    assert!(
        output.status.success(),
        "status={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("class `models.user.User`"));
    assert!(!stdout.contains("HomeView"));
}

#[test]
fn multiple_files_with_stdout_errors() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::write(dir.path().join("a.py"), "class A: ...\n").expect("write a.py");
    std::fs::write(dir.path().join("b.py"), "class B: ...\n").expect("write b.py");

    let output = Command::new(exe)
        .arg(dir.path().to_string_lossy().to_string())
        .arg("--multiple-files")
        .arg("--output")
        .arg("-")
        .output()
        .expect("run pymermaider");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--multiple-files"));
}

#[test]
fn invalid_exclude_glob_errors_cleanly_instead_of_panicking() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::write(dir.path().join("a.py"), "class A: ...\n").expect("write a.py");

    let output = Command::new(exe)
        .arg(dir.path().to_string_lossy().to_string())
        .arg("--exclude")
        .arg("[abc")
        .arg("--output")
        .arg("-")
        .output()
        .expect("run pymermaider");

    assert!(
        !output.status.success(),
        "expected a non-zero exit status for an invalid --exclude glob"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked at"),
        "stderr should not contain a panic backtrace: {stderr}"
    );
    assert!(
        !stderr.contains("RUST_BACKTRACE"),
        "stderr should not contain a panic backtrace: {stderr}"
    );
    assert!(
        stderr.contains("error"),
        "stderr should contain a clean error message: {stderr}"
    );
}

#[test]
fn no_title_multiple_files_still_names_output_files() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    let out_dir = tempfile::TempDir::new().expect("out dir");
    std::fs::write(dir.path().join("a.py"), "class A: ...\n").expect("write a.py");
    std::fs::write(dir.path().join("b.py"), "class B: ...\n").expect("write b.py");

    let output = Command::new(exe)
        .arg(dir.path().to_string_lossy().to_string())
        .arg("--multiple-files")
        .arg("--no-title")
        .arg("--output-dir")
        .arg(out_dir.path().to_string_lossy().to_string())
        .output()
        .expect("run pymermaider");

    assert!(output.status.success());
    let a = std::fs::read_to_string(out_dir.path().join("a.py.md")).expect("a.py.md");
    assert!(a.contains("class A"));
    assert!(!a.contains("title:"));
    assert!(out_dir.path().join("b.py.md").exists());
}

#[test]
fn syntax_errors_are_reported_on_stderr() {
    let exe = env!("CARGO_BIN_EXE_pymermaider");

    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::write(
        dir.path().join("bad.py"),
        "class A:\n    x: int\n\ndef broken(:\n",
    )
    .expect("write bad.py");

    let output = Command::new(exe)
        .arg(dir.path().to_string_lossy().to_string())
        .arg("--output")
        .arg("-")
        .output()
        .expect("run pymermaider");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("syntax error"), "{stderr}");
    assert!(stderr.contains("line 4"), "{stderr}");
    // Valid parts are still rendered
    assert!(String::from_utf8_lossy(&output.stdout).contains("bad.A"));
}

/// Run pymermaider on `file` (stdout output) and return stdout.
#[cfg(feature = "infer")]
fn render(file: &std::path::Path, extra: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_pymermaider"))
        .arg(file)
        .args(extra)
        .args(["--output", "-"])
        .output()
        .expect("run pymermaider");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(feature = "infer")]
#[test]
fn infers_types_from_sibling_modules() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::write(dir.path().join("models.py"), "class Database: ...\n").unwrap();
    let app = dir.path().join("app.py");
    std::fs::write(
        &app,
        "from models import Database\n\nclass Service:\n    def __init__(self):\n        self.db = Database()\n",
    )
    .unwrap();

    let out = render(&app, &[]);
    assert!(out.contains("Database db"), "{out}");
    assert!(out.contains("Service *-- `models.Database`"), "{out}");

    let out = render(&app, &["--no-infer"]);
    assert!(out.contains("Any db"), "{out}");
}

#[cfg(feature = "infer")]
#[test]
fn infers_types_from_third_party_packages_in_a_venv() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let root = dir.path();
    std::fs::write(
        root.join("pyproject.toml"),
        "[project]\nname = \"demo\"\nversion = \"0.0.1\"\n",
    )
    .unwrap();

    // A minimal virtual environment with one installed package.
    let venv = root.join(".venv");
    let site_packages = venv.join("lib/python3.12/site-packages");
    std::fs::create_dir_all(site_packages.join("fancylib")).unwrap();
    std::fs::write(
        venv.join("pyvenv.cfg"),
        "home = /usr/bin\nversion = 3.12.0\n",
    )
    .unwrap();
    std::fs::write(
        site_packages.join("fancylib/__init__.py"),
        "class Widget: ...\n",
    )
    .unwrap();

    std::fs::create_dir_all(root.join("src")).unwrap();
    let app = root.join("src/app.py");
    std::fs::write(
        &app,
        "from fancylib import Widget\n\nclass Ui:\n    def __init__(self):\n        self.w = Widget()\n",
    )
    .unwrap();

    let out = render(&app, &[]);
    assert!(out.contains("Widget w"), "{out}");
    assert!(out.contains("Ui *-- `fancylib.Widget`"), "{out}");
}
