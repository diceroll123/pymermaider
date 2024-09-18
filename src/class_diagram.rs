extern crate env_logger;

use crate::ast;
use crate::checker::Checker;
use crate::parameter_generator::ParameterGenerator;
use itertools::Itertools;
use ruff_python_ast::{Expr, Number};
use ruff_python_semantic::analyze::visibility::{
    is_classmethod, is_overload, is_override, is_staticmethod,
};

const TAB: &str = "    ";

fn normalize_name(name: &str) -> String {
    // make sure name is alphanumeric (including unicode), underscores, and dashes
    // if it's not, then return it with backticks
    if name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        name.to_string()
    } else {
        format!("`{}`", name)
    }
}

pub struct ClassDiagram {
    pub classes: Vec<String>,
    pub relationships: Vec<String>,
    pub path: String,
}

impl ClassDiagram {
    pub fn new() -> Self {
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

        res = res.trim_end().to_string();

        res.push_str("\n```\n");

        res
    }

    pub fn add_class(&mut self, checker: &Checker, class: &ast::StmtClassDef, indent_level: usize) {
        let class_name = class.name.to_string();
        let mut res = String::new();

        let use_tab = TAB.repeat(indent_level);

        res.push_str(&use_tab);
        res.push_str(&format!("class {} {{\n", &class_name));
        for stmt in class.body.iter() {
            res.push_str(&self.process_stmt(checker, stmt, indent_level + 1));
        }
        res.push_str(&use_tab);
        res.push('}');

        self.classes.push(res);

        for base in class.bases() {
            let Some(base) = checker.semantic().resolve_qualified_name(base) else {
                continue;
            };

            let base_name = base.to_string();

            let relationship = format!(
                "{}{} --|> {}\n",
                use_tab,
                class_name,
                normalize_name(&base_name)
            );

            self.relationships.push(relationship);
        }
    }

    fn process_stmt(&self, checker: &Checker, stmt: &ast::Stmt, indent_level: usize) -> String {
        match stmt {
            ast::Stmt::AnnAssign(ast::StmtAnnAssign {
                target,
                annotation,
                simple,
                ..
            }) => {
                let mut res = String::new();
                if !simple {
                    return res;
                }

                let Expr::Name(ast::ExprName { id: target, .. }) = target.as_ref() else {
                    return res;
                };

                let target_name = target.to_string();

                let annotation_name = checker.generator().expr(annotation.as_ref());

                res.push_str(TAB.repeat(indent_level).as_str());

                let is_private = target_name.starts_with('_');

                res.push_str(&format!(
                    "{} {} {}\n",
                    if is_private { "-" } else { "+" },
                    annotation_name,
                    target_name,
                ));

                res
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

                let mut param_gen = ParameterGenerator::new();
                param_gen.unparse_parameters(parameters);

                let params = param_gen.generate();

                let returns = match returns {
                    Some(target) => checker.generator().expr(target.as_ref()),
                    None => "".to_string(),
                };

                let mut method_types = vec![];
                if is_classmethod(decorator_list, checker.semantic()) {
                    method_types.push("@classmethod ");
                } else if is_staticmethod(decorator_list, checker.semantic()) {
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
                    "{} {}{}{}({}) {}\n",
                    if is_private { "-" } else { "+" },
                    method_types.join(""),
                    if *is_async { "async " } else { "" },
                    &name,
                    params,
                    returns,
                ));
                res
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

                res
            }

            _ => "".to_string(),
        }
    }

    pub fn write_to_file(&self, output_directory: &str) -> bool {
        if self.is_empty() {
            info!("No classes found for {0:?}.", self.path);
            return false;
        }

        let path = format!("{output_directory}/{0}.md", self.path);
        if let Some(parent_dir) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }
        std::fs::write(&path, self.render()).unwrap();
        println!("Mermaid file written to: {:?}", path);

        true
    }
}
