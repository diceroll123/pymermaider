extern crate env_logger;
use crate::ast;
use crate::checker::Checker;
use crate::parameter_generator::ParameterGenerator;
use crate::utils::is_abc_class;
use itertools::Itertools as _;
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

        let mut res = String::new();
        res.push('[');
        res.push('"');
        res.push_str(&self.name);
        res.push_str(&post_name_labels.join(""));
        res.push('"');
        res.push(']');

        Some(res)
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

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.relationships.is_empty()
    }

    pub fn render(&self) -> String {
        let mut res = String::new();
        res.push_str("```mermaid\n");

        if !self.path.is_empty() {
            res.push_str("---\n");
            res.push_str(&format!("title: {}\n", self.path));
            res.push_str("---\n");
        }

        res.push_str("classDiagram\n");

        for class in self.classes.iter().unique() {
            res.push_str(class);
            res.push_str("\n\n");
        }

        for relationship in self.relationships.iter().unique() {
            res.push_str(relationship);
            res.push('\n');
        }

        res = res.trim_end().to_owned();

        res.push_str("\n```\n");

        res
    }

    pub fn add_class(&mut self, checker: &Checker, class: &ast::StmtClassDef, indent_level: usize) {
        let class_name = class.name.to_string();
        let mut res = String::new();

        let use_tab = TAB.repeat(indent_level);

        res.push_str(&use_tab);
        res.push_str(&format!("class {} ", &class_name));

        if let Some(label) = class.label(checker) {
            res.push_str(&label);
        }

        let processed_stmts: Vec<String> = class
            .body
            .iter()
            .filter_map(|stmt| self.process_stmt(checker, stmt, indent_level + 1))
            .unique()
            .collect();

        let class_annotation = if class.is_abstract(checker.semantic()) {
            "<<abstract>>"
        } else if class.is_enum(checker.semantic()) {
            "<<enumeration>>"
        } else if class.is_final(checker.semantic()) {
            "<<final>>"
        } else {
            ""
        };

        if !processed_stmts.is_empty() || !class_annotation.is_empty() {
            res.push('{');
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

        for base in class.bases() {
            let base_name = if let Some(base_name) = checker.semantic().resolve_qualified_name(base)
            {
                base_name
            } else {
                let name = checker.locator().slice(base);
                QualifiedName::user_defined(name)
            };

            // skip "object" base class, it's implied
            if matches!(base_name.segments(), ["" | "builtins", "object"]) {
                continue;
            }

            // skip ABCs, they're marked as abstract
            if matches!(base_name.segments(), ["abc", "ABC" | "ABCMeta"])
                && class.is_abstract(checker.semantic())
            {
                continue;
            }

            // skip enums classes
            if class.is_enum(checker.semantic()) {
                continue;
            }

            let relationship = format!(
                "{}{} --|> {}\n",
                use_tab,
                class_name,
                base_name.normalize_name()
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

                let annotation_name = checker.generator().expr(annotation.as_ref());

                res.push_str(TAB.repeat(indent_level).as_str());

                let is_private = target_name.starts_with('_');

                res.push_str(&format!(
                    "{} {} {}\n",
                    if is_private { '-' } else { '+' },
                    annotation_name,
                    target_name,
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
                    &name,
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

                    res.push_str(&TAB.repeat(indent_level));
                    res.push_str("+ ");
                    if !value_type.is_empty() {
                        res.push_str(value_type);
                        res.push(' ');
                    }
                    res.push_str(&format!("{target_name}\n"));
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

        let expected_output = "```mermaid
classDiagram
    class TestClass {
        - __init__(self, x, y) None
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
    class Thing ["Thing[T]"]
```"#;

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_inner_class() {
        let source = "
class Thing(Inner[T]): ...
";

        let expected_output = "```mermaid
classDiagram
    class Thing

    Thing --|> `Inner[T]`
```";

        test_diagram(source, expected_output);
    }

    #[test]
    fn test_class_diagram_generic_class_multiple() {
        let source = "
class Thing[T, U, V]: ...
";

        let expected_output = r#"```mermaid
classDiagram
    class Thing ["Thing[T, U, V]"]
```
"#;

        test_diagram(source, expected_output);
        // test_diagram_print(source);
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

    ItemBase --|> `pydantic.BaseModel`

    ItemCreate --|> ItemBase

    Item --|> ItemBase

    UserBase --|> `pydantic.BaseModel`

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

        let expected_output = "```mermaid
classDiagram
    class Thing {
        - @overload __init__(self, x, y) None
        - __init__(self, x, y) None
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

        let expected_output = "```mermaid
classDiagram
    class Thing {
        - __complex__(self) complex
        - __bytes__(self) bytes
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
