use crate::ast;
use crate::checker::Checker;
use crate::parameter_generator::ParameterGenerator;
use crate::utils::is_abc_class;
use itertools::Itertools as _;
#[cfg(feature = "cli")]
use log::info;
use ruff_linter::source_kind::SourceKind;
use ruff_linter::Locator;
use ruff_python_ast::name::QualifiedName;
use ruff_python_ast::{Arguments, Expr, Number, PySourceType};
use ruff_python_codegen::Stylist;
use ruff_python_parser::parse_unchecked_source;
use ruff_python_semantic::analyze::class::is_enumeration;
use ruff_python_semantic::analyze::visibility::{
    is_abstract, is_classmethod, is_final, is_overload, is_override, is_staticmethod,
};
use ruff_python_semantic::{Module, ModuleKind, ModuleSource, SemanticModel};
use ruff_python_stdlib::typing::simple_magic_return_type;
use std::path::{Path, PathBuf};

const TAB: &str = "    ";

/// Escape leading underscores for Mermaid diagrams
/// Mermaid interprets __ as formatting, so we escape leading underscores with backslashes
fn escape_underscores(s: &str) -> String {
    let leading_underscores = s.chars().take_while(|&c| c == '_').count();
    if leading_underscores > 0 {
        format!(
            "{}{}",
            r"\_".repeat(leading_underscores),
            &s[leading_underscores..]
        )
    } else {
        s.to_string()
    }
}

trait QualifiedNameDiagramHelpers {
    fn normalize_name(&self) -> String;
}

impl QualifiedNameDiagramHelpers for QualifiedName<'_> {
    fn normalize_name(&self) -> String {
        // make sure name is alphanumeric (including unicode), underscores, and dashes
        // if it's not, then return it with backticks
        let name = self.to_string();
        if name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            name
        } else {
            format!("`{name}`")
        }
    }
}

trait ClassDefDiagramHelpers {
    fn is_abstract(&self, semantic: &SemanticModel) -> bool;
    fn is_final(&self, semantic: &SemanticModel) -> bool;
    fn is_enum(&self, semantic: &SemanticModel) -> bool;
    fn label(&self, checker: &Checker) -> Option<String>;
}

impl ClassDefDiagramHelpers for ast::StmtClassDef {
    fn label(&self, checker: &Checker) -> Option<String> {
        let mut post_name_labels = vec![];

        if let Some(params) = &self.type_params {
            post_name_labels.push(checker.locator().slice(params.as_ref()));
        }

        if post_name_labels.is_empty() {
            return None;
        }

        // Extract the actual types without brackets
        let type_params = post_name_labels.join("");
        let clean_params = type_params.trim_start_matches('[').trim_end_matches(']');

        // Format with tildes instead of brackets, with a space
        Some(format!(" ~{}~", clean_params))
    }

    fn is_abstract(&self, semantic: &SemanticModel) -> bool {
        let Some(Arguments { args, keywords, .. }) = self.arguments.as_deref() else {
            return false;
        };

        if args.len() + keywords.len() != 1 {
            return false;
        }

        is_abc_class(args, keywords, semantic)
    }

    fn is_enum(&self, semantic: &SemanticModel) -> bool {
        is_enumeration(self, semantic)
    }

    fn is_final(&self, semantic: &SemanticModel) -> bool {
        is_final(&self.decorator_list, semantic)
    }
}

// Helper function to check if a base is a Generic type
fn is_generic_base(base_name: &QualifiedName) -> bool {
    matches!(base_name.segments(), ["typing", "Generic"])
}

// Helper function to extract generic type info from bases
fn extract_generic_info(base: &Expr, checker: &Checker) -> Option<String> {
    // Must be a subscript expression (like Generic[T])
    let Expr::Subscript(subscript) = base else {
        return None;
    };

    // Must be a qualified name that resolves to typing.Generic
    let base_name = checker
        .semantic()
        .resolve_qualified_name(&subscript.value)?;

    if !is_generic_base(&base_name) {
        return None;
    }

    // Get the type string
    let type_var = checker.locator().slice(base).to_string();

    // For type parameter extraction, return just the parameter without Generic[]
    let start_idx = type_var.find('[').map(|idx| idx + 1)?;
    let end_idx = type_var.rfind(']')?;

    if start_idx < end_idx {
        Some(type_var[start_idx..end_idx].trim().to_string())
    } else {
        None
    }
}

pub struct ClassDiagram {
    classes: Vec<String>,
    relationships: Vec<String>,
    pub path: String,
}

impl ClassDiagram {
    pub const fn new() -> Self {
        Self {
            classes: vec![],
            relationships: vec![],
            path: String::new(),
        }
    }

    #[cfg(feature = "cli")]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.relationships.is_empty()
    }

    pub fn render(&self) -> String {
        let mut res = String::with_capacity(1024); // Pre-allocate a reasonable size
        res.push_str("```mermaid\n");

        if !self.path.is_empty() {
            res.push_str("---\n");
            res.push_str(&format!("title: {}\n", self.path));
            res.push_str("---\n");
        }

        res.push_str("classDiagram\n");

        // Add classes
        for class in self.classes.iter().unique() {
            res.push_str(class);
            res.push_str("\n\n");
        }

        // Add relationships (no extra newlines between them)
        let mut unique_relationships = self.relationships.iter().unique();
        res.push_str(&unique_relationships.join("\n"));

        res = res.trim_end().to_owned();

        res.push_str("\n```\n");

        res
    }

    pub fn add_class(&mut self, checker: &Checker, class: &ast::StmtClassDef, indent_level: usize) {
        let class_name = class.name.to_string();
        let mut res = String::new();
        let use_tab = TAB.repeat(indent_level);

        // Find generic type parameters from bases (for Generic[T])
        let mut generic_type_var = None;
        for base in class.bases() {
            if let Some(type_var) = extract_generic_info(base, checker) {
                generic_type_var = Some(type_var);
                break;
            }
        }

        // Build class name with optional type parameters
        res.push_str(&use_tab);
        res.push_str(&format!("class {}", &class_name));
        if let Some(label) = class.label(checker) {
            // Already has explicit type parameters via [T] syntax
            res.push_str(&label);
        } else if let Some(ref type_var) = generic_type_var {
            // Has Generic[T] parameter
            res.push_str(&format!(" ~{}~", type_var));
        }

        // Process class body statements
        let processed_stmts: Vec<String> = class
            .body
            .iter()
            .filter_map(|stmt| self.process_stmt(checker, stmt, indent_level + 1))
            .unique()
            .collect();

        // Determine if this class is abstract, an enum, or final
        let is_abstract = class.is_abstract(checker.semantic())
            || class.bases().iter().any(|base| {
                checker
                    .semantic()
                    .resolve_qualified_name(base)
                    .is_some_and(|name| {
                        matches!(
                            name.segments(),
                            ["abc", "ABC" | "ABCMeta"] | ["typing", "ABC"]
                        )
                    })
            });

        let class_annotation = if is_abstract {
            "<<abstract>>"
        } else if class.is_enum(checker.semantic()) {
            "<<enumeration>>"
        } else if class.is_final(checker.semantic()) {
            "<<final>>"
        } else {
            ""
        };

        // Add class body with any annotations
        if !processed_stmts.is_empty() || !class_annotation.is_empty() {
            res.push_str(" {");
            res.push('\n');

            if !class_annotation.is_empty() {
                res.push_str(&use_tab);
                res.push_str(&use_tab);
                res.push_str(class_annotation);
                res.push('\n');
            }

            for stmt in processed_stmts {
                res.push_str(&stmt);
            }

            res.push_str(&use_tab);
            res.push('}');
        }

        self.classes.push(res.trim_end().to_owned());

        // Handle inheritance relationships
        let relation_symbol = if is_abstract { "..|>" } else { "--|>" };

        for base in class.bases() {
            // skip if it's a generic type or a built-in type or an ABC or an enum
            let should_skip = extract_generic_info(base, checker).is_some()
                || checker
                    .semantic()
                    .resolve_qualified_name(base)
                    .is_some_and(|name| {
                        matches!(name.segments(), ["typing", "Generic"])
                            || matches!(name.segments(), ["" | "builtins", "object"])
                            || matches!(name.segments(), ["abc", "ABC" | "ABCMeta"])
                            || matches!(name.segments(), ["typing", "ABC"])
                    })
                || class.is_enum(checker.semantic());

            if should_skip {
                continue;
            }

            let base_name = match checker.semantic().resolve_qualified_name(base) {
                Some(base_name) => base_name.normalize_name(),
                None => {
                    let name = checker.locator().slice(base);
                    QualifiedName::user_defined(name).normalize_name()
                }
            };

            // Extract just the base class name without the generic specialization
            let base_display = if base_name.contains('[') {
                base_name
                    .split('[')
                    .next()
                    .unwrap_or(&base_name)
                    .to_string()
            } else {
                base_name
            }
            .trim_matches('`')
            .to_string();

            let relationship = format!(
                "{}{} {} {}\n",
                use_tab, class_name, relation_symbol, base_display
            );

            self.relationships.push(relationship);
        }
    }

    fn process_stmt(
        &self,
        checker: &Checker,
        stmt: &ast::Stmt,
        indent_level: usize,
    ) -> Option<String> {
        match stmt {
            ast::Stmt::AnnAssign(ast::StmtAnnAssign {
                target,
                annotation,
                simple,
                ..
            }) => {
                let mut res = String::new();
                if !simple {
                    return None;
                }

                let Expr::Name(ast::ExprName { id: target, .. }) = target.as_ref() else {
                    return None;
                };

                let target_name = target.to_string();
                let escaped_target_name = escape_underscores(&target_name);

                let annotation_name = checker.generator().expr(annotation.as_ref());

                res.push_str(TAB.repeat(indent_level).as_str());

                let is_private = target_name.starts_with('_');

                res.push_str(&format!(
                    "{} {} {}\n",
                    if is_private { '-' } else { '+' },
                    annotation_name,
                    escaped_target_name,
                ));

                Some(res)
            }

            ast::Stmt::FunctionDef(ast::StmtFunctionDef {
                name,
                is_async,
                parameters,
                returns,
                decorator_list,
                ..
            }) => {
                let is_private = name.starts_with('_');
                let escaped_name = escape_underscores(name.as_str());

                let is_static = is_staticmethod(decorator_list, checker.semantic());

                let mut param_gen = ParameterGenerator::new();
                param_gen.unparse_parameters(parameters);

                let params = param_gen.generate();

                let returns = match returns {
                    Some(target) => checker.generator().expr(target.as_ref()),
                    None => String::new(),
                };

                let mut method_types = vec![];

                if is_final(decorator_list, checker.semantic()) {
                    method_types.push("@final ");
                }

                if is_classmethod(decorator_list, checker.semantic()) {
                    method_types.push("@classmethod ");
                } else if is_static {
                    method_types.push("@staticmethod ");
                }

                if is_overload(decorator_list, checker.semantic()) {
                    method_types.push("@overload ");
                }

                if is_override(decorator_list, checker.semantic()) {
                    method_types.push("@override ");
                }

                let mut res = String::new();
                res.push_str(&TAB.repeat(indent_level));
                res.push_str(&format!(
                    "{} {}{}{}({})",
                    if is_private { '-' } else { '+' },
                    method_types.join(""),
                    if *is_async { "async " } else { "" },
                    &escaped_name,
                    params,
                ));

                if !returns.is_empty() {
                    res.push_str(&format!(" {returns}"));
                } else if let Some(method) = simple_magic_return_type(name) {
                    res.push_str(&format!(" {method}"));
                }

                // Mermaid diagrams don't support multiple of these classifiers at this time
                if is_abstract(decorator_list, checker.semantic()) {
                    res.push('*');
                } else if is_static {
                    res.push('$');
                }

                res.push('\n');
                Some(res)
            }

            ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) => {
                let mut res = String::new();

                let value_type = match value.as_ref() {
                    Expr::BoolOp(_) => "bool",
                    Expr::BinOp(_) | Expr::UnaryOp(_) => "int",
                    Expr::Lambda(_) => "Callable",
                    Expr::DictComp(_) | Expr::Dict(_) => "dict",
                    Expr::Set(_) | Expr::SetComp(_) => "set",
                    Expr::FString(_) | Expr::StringLiteral(_) => "str",
                    Expr::NoneLiteral(_) => "None",
                    Expr::BooleanLiteral(_) => "bool",
                    Expr::BytesLiteral(_) => "bytes",
                    Expr::EllipsisLiteral(_) => "...",
                    Expr::ListComp(_) | Expr::List(_) => "list",
                    Expr::Tuple(_) => "tuple",
                    Expr::NumberLiteral(inner) => match inner.value {
                        Number::Int(_) => "int",
                        Number::Float(_) => "float",
                        Number::Complex { .. } => "complex",
                    },
                    _ => "",
                };

                for target in targets {
                    let Expr::Name(ast::ExprName { id: target, .. }) = target else {
                        continue;
                    };

                    let target_name = target.to_string();
                    let escaped_target_name = escape_underscores(&target_name);

                    res.push_str(&TAB.repeat(indent_level));
                    res.push_str("+ ");
                    if !value_type.is_empty() {
                        res.push_str(value_type);
                        res.push(' ');
                    }
                    res.push_str(&format!("{escaped_target_name}\n"));
                }

                if res.is_empty() {
                    None
                } else {
                    Some(res)
                }
            }

            _ => None,
        }
    }

    #[cfg(feature = "cli")]
    #[allow(dead_code)]
    pub fn write_to_file(&self, output_directory: &Path) -> bool {
        if self.is_empty() {
            info!("No classes found for {0:?}.", self.path);
            return false;
        }

        let path = format!("{0}/{1}.md", output_directory.to_string_lossy(), self.path);
        if let Some(parent_dir) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }
        std::fs::write(&path, self.render()).unwrap();
        println!("Mermaid file written to: {path:?}");

        true
    }

    pub fn add_to_diagram(&mut self, source: String, file: &PathBuf) {
        let source_type = PySourceType::from(file);
        let source_kind = SourceKind::Python(source);

        let locator = Locator::new(source_kind.source_code());

        let parsed = parse_unchecked_source(source_kind.source_code(), source_type);

        let stylist = Stylist::from_tokens(parsed.tokens(), source_kind.source_code());

        let python_ast = parsed.into_suite();

        let kind = if file.ends_with("__init__.py") {
            ModuleKind::Package
        } else {
            ModuleKind::Module
        };

        let module = Module {
            kind,
            source: ModuleSource::File(Path::new(file)),
            python_ast: &python_ast,
            name: None,
        };
        let semantic = SemanticModel::new(&[], Path::new(file), module);
        let mut checker = Checker::new(&stylist, &locator, semantic);
        checker.see_imports(&python_ast);

        for stmt in &python_ast {
            if let ast::Stmt::ClassDef(class) = stmt {
                // we only care about class definitions
                self.add_class(&checker, class, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_diagram_basic() {
        let source = "
class TestClass:
    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y
    def add(self, x: int, y: int) -> int:
        return x + y
    def subtract(self, x: int, y: int) -> int:
        return x - y
";

        let expected_output = r"```mermaid
classDiagram
    class TestClass {
        - \_\_init__(self, x, y) None
        + add(self, x, y) int
        + subtract(self, x, y) int
    }
```
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_class() {
        let source = "
class Thing[T]: ...
";

        let expected_output = r#"```mermaid
classDiagram
    class Thing ~T~
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_inner_class() {
        let source = "
class Thing(Inner[T]): ...
";

        let expected_output = r#"```mermaid
classDiagram
    class Thing

    Thing --|> Inner
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic() {
        let source = r#"
from typing import TypeVar, Generic
from abc import ABC
FancyType = TypeVar("FancyType")
class Thing(ABC, Generic[FancyType]): ...
"#;

        let expected_output = r#"```mermaid
classDiagram
    class Thing ~FancyType~ {
        <<abstract>>
    }
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_class_multiple() {
        let source = "
class Thing[T, U, V]: ...
";

        let expected_output = r#"```mermaid
classDiagram
    class Thing ~T, U, V~
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_final() {
        let source = "
from typing import final
@final
class Thing: ...
";

        let expected_output = "```mermaid
classDiagram
    class Thing {
        <<final>>
    }
```
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_ellipsis() {
        let source = "
class Thing: ...
";

        let expected_output = "```mermaid
classDiagram
    class Thing
```
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_complex() {
        // this tests async, classmethod, args, return type
        let source = "
class Thing:
    @classmethod
    async def foo(cls, first, /, *second, kwarg: bool = True, **unpack_this) -> dict[str, str]: ...
";

        let expected_output = "```mermaid
classDiagram
    class Thing {
        + @classmethod async foo(cls, first, /, *second, kwarg, **unpack_this) dict[str, str]
    }
```
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_pydantic_example() {
        let source = "
from pydantic import BaseModel


class ItemBase(BaseModel):
    title: str
    description: str | None = None


class ItemCreate(ItemBase):
    pass


class Item(ItemBase):
    id: int
    owner_id: int

    class Config:
        orm_mode = True


class UserBase(BaseModel):
    email: str


class UserCreate(UserBase):
    password: str


class User(UserBase):
    id: int
    is_active: bool
    items: list[Item] = []

    class Config:
        orm_mode = True
";

        let expected_output = "```mermaid
classDiagram
    class ItemBase {
        + str title
        + str | None description
    }

    class ItemCreate

    class Item {
        + int id
        + int owner_id
    }

    class UserBase {
        + str email
    }

    class UserCreate {
        + str password
    }

    class User {
        + int id
        + bool is_active
        + list[Item] items
    }

    ItemBase --|> pydantic.BaseModel

    ItemCreate --|> ItemBase

    Item --|> ItemBase

    UserBase --|> pydantic.BaseModel

    UserCreate --|> UserBase

    User --|> UserBase
```
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_unique_overloads() {
        let source = "
from typing import overload
class Thing:
    @overload
    def __init__(self, x: int, y: int) -> None: ...

    @overload
    def __init__(self, x: str, y: str) -> None: ...

    def __init__(self, x: int | str, y: int | str) -> None: ...
";

        let expected_output = r"```mermaid
classDiagram
    class Thing {
        - @overload \_\_init__(self, x, y) None
        - \_\_init__(self, x, y) None
    }
```
";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_object_base() {
        let source = "
class Thing(object): ...
";

        let expected_output = "```mermaid
classDiagram
    class Thing
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_dundermagic_infer() {
        let source = "
class Thing:
    def __complex__(self): ...
    def __bytes__(self): ...
";

        let expected_output = r"```mermaid
classDiagram
    class Thing {
        - \_\_complex__(self) complex
        - \_\_bytes__(self) bytes
    }
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_notimplemented() {
        let source = "
class Thing:
    def do_thing(self):
        raise NotImplementedError
";

        let expected_output = "```mermaid
classDiagram
    class Thing {
        + do_thing(self)
    }
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_abstract_base_class() {
        let source = r#"
from abc import ABC, abstractmethod
class Thing(ABC):
    @abstractmethod
    def do_thing(self) -> None:
        """Must be implemented by subclasses"""
        pass
"#;
        let expected_output = "```mermaid
classDiagram
    class Thing {
        <<abstract>>
        + do_thing(self) None*
    }
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_enum() {
        let source = "
from enum import Enum
class Color(Enum):
    RED = 1
    GREEN = 2
    BLUE = 3
";

        let expected_output = "```mermaid
classDiagram
    class Color {
        <<enumeration>>
        + int RED
        + int GREEN
        + int BLUE
    }
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_staticmethod() {
        let source = "
class Thing:
    @staticmethod
    def static_method(x: int, y: int) -> int:
        return x + y
";
        let expected_output = "```mermaid
classDiagram
    class Thing {
        + @staticmethod static_method(x, y) int$
    }
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_concrete_generic_base() {
        let source = r#"
from typing import TypeVar, Generic
IndexType = TypeVar("IndexType")

class Store(Generic[IndexType]):
    def insert(self, data) -> None:
        pass

class MemoryStore(Store[int]):
    def insert(self, data) -> None:
        self.storage.append(data)
"#;

        let expected_output = r#"```mermaid
classDiagram
    class Store ~IndexType~ {
        + insert(self, data) None
    }

    class MemoryStore {
        + insert(self, data) None
    }

    MemoryStore --|> Store
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_abstract_generic_inheritance() {
        let source = r#"
from typing import TypeVar, Generic
from abc import ABC, abstractmethod
IndexType = TypeVar("IndexType")

class Store(ABC, Generic[IndexType]):
    @abstractmethod
    def insert(self, data) -> None:
        pass

class MemoryStore(Store[int]):
    def insert(self, data) -> None:
        self.storage.append(data)
"#;

        let expected_output = r#"```mermaid
classDiagram
    class Store ~IndexType~ {
        <<abstract>>
        + insert(self, data) None*
    }

    class MemoryStore {
        + insert(self, data) None
    }

    MemoryStore --|> Store
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_full_generics_example() {
        let source = r#"
from typing import TypeVar, Generic
from abc import ABC, abstractmethod
from datetime import datetime

IndexType = TypeVar("IndexType")
FancyStorage = TypeVar("FancyStorage")

class Store(ABC, Generic[IndexType]):
    @abstractmethod
    def insert(self, data) -> None:
        pass

class MemoryStore(Store[datetime]):
    def insert(self, data) -> None:
        self.storage.append(data)

class FancyStore(Store[datetime], Generic[FancyStorage]):
    def __init__(self, fancy_store: FancyStorage) -> None:
        self.storage = fancy_store

    def insert(self, data) -> None:
        self.storage.insert(data)
"#;

        let expected_output = r"```mermaid
classDiagram
    class Store ~IndexType~ {
        <<abstract>>
        + insert(self, data) None*
    }

    class MemoryStore {
        + insert(self, data) None
    }

    class FancyStore ~FancyStorage~ {
        - \_\_init__(self, fancy_store) None
        + insert(self, data) None
    }

    MemoryStore --|> Store

    FancyStore --|> Store
```";

        test_diagram(source, expected_output);
    }

    fn test_diagram(source: &str, expected_output: &str) {
        let mut diagram = ClassDiagram::new();
        diagram.add_to_diagram(source.to_owned(), &PathBuf::from("test.py"));
        let output = diagram.render();
        assert_eq!(output.trim(), expected_output.trim());
    }

    #[expect(dead_code)]
    fn test_diagram_print(source: &str) {
        // for making new tests and debugging :P
        let mut diagram = ClassDiagram::new();
        diagram.add_to_diagram(source.to_owned(), &PathBuf::from("test.py"));
        println!("{}", diagram.render());
        assert_eq!(1, 2);
    }
}
