/// ty-backed type inference (feature `infer`).
///
/// ty infers types on its own parsed AST, so nodes from our parse can't be passed in. Instead the
/// source is analyzed once and the inferred type of every assignment target is recorded by its
/// source range, which is identical in both parses of the same text.
use std::collections::HashMap;

use ruff_db::files::{system_path_to_file, File};
use ruff_db::parsed::parsed_module;
use ruff_db::system::{InMemorySystem, SystemPath, SystemPathBuf};
use ruff_python_ast::name::Name;
use ruff_python_ast::visitor::source_order::{walk_stmt, SourceOrderVisitor, TraversalSignal};
use ruff_python_ast::{AnyNodeRef, Expr, Stmt};
use ruff_text_size::{Ranged, TextRange};
use ty_project::{ProjectDatabase, ProjectMetadata};
use ty_python_semantic::{Db as _, HasType, SemanticModel};

/// A ty project rooted on the real filesystem, shared by every file in a run.
///
/// Unlike [`TyInferer::new`], ty can see sibling modules and the project's virtual environment
/// (`VIRTUAL_ENV`, `.venv`, or the configured interpreter), so types from other files and
/// third-party packages resolve. Not available on wasm, which has no filesystem.
#[cfg(not(target_arch = "wasm32"))]
pub struct TyProject {
    db: std::sync::Mutex<ProjectDatabase>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TyProject {
    /// Discover the ty project for `root` (honoring `pyproject.toml` / `ty.toml`).
    /// Returns `None` if discovery fails, in which case callers fall back to [`TyInferer::new`].
    pub fn discover(root: &std::path::Path) -> Option<Self> {
        use ruff_db::system::OsSystem;

        let cwd = SystemPathBuf::from_path_buf(std::env::current_dir().ok()?).ok()?;
        let mut root = std::path::absolute(root).ok()?;
        // The CLI argument may be a single file; ty discovers projects from a directory.
        if root.is_file() {
            root.pop();
        }
        let root = SystemPathBuf::from_path_buf(root).ok()?;
        let system = OsSystem::new(&cwd);
        let mut metadata = ProjectMetadata::discover(&root, &system).ok()?;
        metadata.apply_configuration_files(&system).ok()?;
        let db = ProjectDatabase::fallible(metadata, system).ok()?;
        Some(Self {
            db: std::sync::Mutex::new(db),
        })
    }

    /// Analyze the file at `path`, which must be saved on disk with the same text that is being
    /// diagrammed.
    pub fn infer_file(&self, path: &std::path::Path) -> Option<TyInferer> {
        // Clones share ty's caches, so each thread gets its own handle on the same project.
        let db = self.db.lock().ok()?.clone();
        let path = SystemPathBuf::from_path_buf(std::path::absolute(path).ok()?).ok()?;
        let file = system_path_to_file(&db, &path).ok()?;
        TyInferer::analyze(&db, file)
    }
}

const ROOT: &str = "/";
const FILE: &str = "/source.py";

/// Inferred types for assignment targets in a single source file.
#[derive(Debug, Default)]
pub struct TyInferer {
    targets: HashMap<TextRange, String>,
}

impl TyInferer {
    /// Analyze `source` on its own, in an in-memory project. Only the file itself and the
    /// stdlib are visible, so imports from other modules stay unresolved. Returns `None` if ty
    /// could not be set up.
    pub fn new(source: &str) -> Option<Self> {
        let system = InMemorySystem::default();
        let root = SystemPathBuf::from(ROOT);
        let path = SystemPath::new(FILE);
        system.fs().create_directory_all(&root).ok()?;
        system.fs().write_file_all(path, source).ok()?;

        let project = ProjectMetadata::new(Name::new_static("pymermaider"), root);
        let db = ProjectDatabase::fallible(project, system).ok()?;
        let file = system_path_to_file(&db, path).ok()?;
        Self::analyze(&db, file)
    }

    fn analyze(db: &ProjectDatabase, file: File) -> Option<Self> {
        let program_file = db.program_file(file);
        let model = SemanticModel::new(db, program_file);
        let parsed = parsed_module(db, program_file.python_file(db)).load(db);

        // Types are rendered to strings eagerly so nothing borrows from `db` afterwards.
        let mut collector = Collector {
            model: &model,
            out: HashMap::new(),
        };
        collector.visit_body(parsed.suite());
        Some(Self {
            targets: collector.out,
        })
    }

    /// The inferred type of the assignment target spanning `range`, if known and informative.
    pub fn target_type(&self, range: TextRange) -> Option<&str> {
        self.targets.get(&range).map(String::as_str)
    }
}

/// Turn ty's display text into something suitable for a diagram, or `None` if it isn't a
/// plain type expression (unknown, callables, modules, enum literals, ...).
///
/// Literal types are widened (`Literal[0]` -> `int`) since diagrams show declared-style types.
fn normalize(text: &str) -> Option<String> {
    if text.contains("Unknown") || text.contains(['<', '@']) || text.contains(" -> ") {
        return None;
    }
    let text = match text
        .strip_prefix("Literal[")
        .and_then(|t| t.strip_suffix(']'))
    {
        Some(inner) => {
            let first = inner.split(", ").next().unwrap_or(inner);
            match first.chars().next()? {
                '0'..='9' | '-' => "int",
                '"' | '\'' => "str",
                'b' if first[1..].starts_with(['"', '\'']) => "bytes",
                _ if first == "True" || first == "False" => "bool",
                _ => return None,
            }
        }
        None if text.contains("Literal[") => return None,
        None => text,
    };
    ruff_python_parser::parse_expression(text).ok()?;
    Some(text.to_owned())
}

struct Collector<'a, 'db> {
    model: &'a SemanticModel<'db>,
    out: HashMap<TextRange, String>,
}

impl Collector<'_, '_> {
    fn record(&mut self, target: &Expr) {
        let Some(ty) = target.inferred_type(self.model) else {
            return;
        };
        if ty.is_unknown() {
            return;
        }
        let env = self.model.program_environment();
        let text = ty.display(self.model.db(), &env).to_string();
        if let Some(text) = normalize(&text) {
            self.out.insert(target.range(), text);
        }
    }

    fn visit_body(&mut self, body: &[Stmt]) {
        for stmt in body {
            self.visit_stmt(stmt);
        }
    }
}

impl<'a> SourceOrderVisitor<'a> for Collector<'_, '_> {
    fn enter_node(&mut self, _node: AnyNodeRef<'a>) -> TraversalSignal {
        TraversalSignal::Traverse
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Assign(assign) => assign.targets.iter().for_each(|t| self.record(t)),
            Stmt::AnnAssign(ann) => self.record(&ann.target),
            _ => {}
        }
        walk_stmt(self, stmt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_constructor_call_attribute() {
        let source = "
class Database:
    pass

class Service:
    def __init__(self):
        self.db = Database()
        self.count = 0
";
        let inferer = TyInferer::new(source).expect("ty should initialize");
        let mut found: Vec<&str> = inferer.targets.values().map(String::as_str).collect();
        found.sort_unstable();
        assert_eq!(found, ["Database", "int"]);
    }

    #[test]
    fn normalize_widens_literals_and_rejects_noise() {
        assert_eq!(normalize("Literal[0]").as_deref(), Some("int"));
        assert_eq!(normalize("Literal[1, 2]").as_deref(), Some("int"));
        assert_eq!(normalize("Literal[\"a\"]").as_deref(), Some("str"));
        assert_eq!(normalize("Literal[b\"a\"]").as_deref(), Some("bytes"));
        assert_eq!(normalize("Literal[True]").as_deref(), Some("bool"));
        assert_eq!(normalize("Literal[Color.RED]"), None);
        assert_eq!(normalize("Literal[1] | None"), None);
        assert_eq!(normalize("list[Unknown]"), None);
        assert_eq!(normalize("<class 'Foo'>"), None);
        assert_eq!(normalize("(x: int) -> str"), None);
        assert_eq!(
            normalize("list[Foo] | None").as_deref(),
            Some("list[Foo] | None")
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn project_resolves_sibling_modules() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("models.py"), "class Database:\n    pass\n").unwrap();
        let app = dir.path().join("app.py");
        std::fs::write(
            &app,
            "from models import Database\n\nclass Service:\n    def __init__(self):\n        self.db = Database()\n",
        )
        .unwrap();

        let project = TyProject::discover(dir.path()).expect("project should be discovered");
        let inferer = project.infer_file(&app);
        println!("{inferer:?}");
        let inferer = inferer.expect("file should be analyzed");
        let types: Vec<&str> = inferer.targets.values().map(String::as_str).collect();
        assert_eq!(types, ["Database"]);
    }
}
