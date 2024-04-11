mod checker;
mod class_diagram;
mod parameter_generator;

#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use crate::class_diagram::ClassDiagram;
use crate::parameter_generator::ParameterGenerator;
use ast::{Expr, Number};
use checker::Checker;
use clap::{App, Arg};
use ignore::{types::TypesBuilder, WalkBuilder};
use ruff_python_ast as ast;
use ruff_python_codegen::Stylist;
use ruff_python_parser::lexer::lex;
use ruff_python_parser::{parse_suite, Mode};
use ruff_python_semantic::analyze::visibility::{
    self, is_classmethod, is_overload, is_staticmethod,
};
use ruff_python_semantic::{Module, ModuleKind, SemanticModel};
use ruff_source_file::Locator;
use std::path::Path;

const TAB: &str = "    ";

fn main() {
    env_logger::init();
    let app = App::new("pymermaider")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Converts Python files into Mermaid class diagrams.")
        .arg(
            Arg::with_name("path")
                .help("Path to a file or directory")
                .required(true),
        )
        .arg(
            Arg::with_name("multiple")
                .short("m")
                .long("multiple-files")
                .help("Process each file individually, outputting a mermaid file for each file. Only used when path is a directory.")
                .takes_value(false),
        );

    let matches = app.get_matches();

    let mut written: usize = 0;

    let path = Path::new(matches.value_of("path").unwrap());

    if path.exists() {
        if path.is_file() {
            let title = path.file_name().unwrap().to_str().unwrap();
            let mut diagram = make_mermaid(vec![path.to_str().unwrap().to_string()]);
            diagram.title = title.to_string();

            let wrote_file = diagram.write_to_file(title);
            if wrote_file {
                written += 1;
            }
        } else if path.is_dir() {
            let parsed_files = parse_folder(path).unwrap();

            let multiple_files = matches.is_present("multiple");

            if multiple_files {
                for parsed_file in parsed_files.iter() {
                    let title = Path::new(parsed_file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let mut diagram = make_mermaid(vec![parsed_file.clone()]);
                    diagram.title = title.to_string();

                    let wrote_file = diagram.write_to_file(title);
                    if wrote_file {
                        written += 1;
                    }
                }
            } else {
                let canonical_path = path.canonicalize().unwrap();
                let title = canonical_path.file_name().unwrap().to_str().unwrap();

                let mut diagram = make_mermaid(parsed_files);
                diagram.title = title.to_string();

                let wrote_file = diagram.write_to_file(title);
                if wrote_file {
                    written += 1;
                }
            }
        }
        println!("{} files written.", written);
    } else {
        println!("{:?} does not exist.", path);
    }
}

fn stmt_mermaider(checker: &Checker, stmt: &ast::Stmt, indent_level: usize) -> String {
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
            let function_name = name.to_string();

            let is_private = function_name.starts_with('_');

            let mut param_gen = ParameterGenerator::new();
            param_gen.unparse_parameters(parameters);

            let params = param_gen.generate();

            let returns = match returns {
                Some(target) => checker.generator().expr(target.as_ref()),
                None => "".to_string(),
            };

            let mut method_type = "";
            if is_classmethod(decorator_list, checker.semantic()) {
                method_type = "@classmethod ";
            } else if is_staticmethod(decorator_list, checker.semantic()) {
                method_type = "@staticmethod ";
            } else if is_overload(decorator_list, checker.semantic()) {
                method_type = "@overload ";
            }

            let mut res = String::new();
            res.push_str(&TAB.repeat(indent_level));
            res.push_str(&format!(
                "{} {}{}{}({}) {}\n",
                if is_private { "-" } else { "+" },
                method_type,
                if *is_async { "async " } else { "" },
                &function_name,
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
                Expr::DictComp(_) | ast::Expr::Dict(_) => "dict",
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

fn class_mermaider(
    class_diagram: &mut ClassDiagram,
    checker: &Checker,
    class: &ast::StmtClassDef,
    indent_level: usize,
) {
    let class_name = class.name.to_string();
    let mut res = String::new();

    let use_tab = TAB.repeat(indent_level);

    res.push_str(&use_tab);
    res.push_str(&format!("class {} {{\n", &class_name));
    for stmt in class.body.iter() {
        res.push_str(&stmt_mermaider(checker, stmt, indent_level + 1));
    }
    res.push_str(&use_tab);
    res.push('}');

    class_diagram.classes.push(res);

    for base in class.bases() {
        let Expr::Name(ast::ExprName { id: base, .. }) = base else {
            continue;
        };

        let base_name = base.to_string();

        let relationship = format!("{}{} --|> {}\n", use_tab, class_name, base_name);

        class_diagram.relationships.push(relationship);
    }
}

fn make_mermaid(parsed_files: Vec<String>) -> ClassDiagram {
    let mut class_diagram = ClassDiagram::new();

    for file in parsed_files.iter() {
        let source = match std::fs::read_to_string(file) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let locator = Locator::new(&source);
        let tokens: Vec<_> = lex(&source, Mode::Module).collect();
        let stylist = Stylist::from_tokens(&tokens, &locator);
        let python_ast = parse_suite(&source).unwrap();
        let module = Module {
            kind: ModuleKind::Module,
            source: visibility::ModuleSource::File(Path::new(file)),
            python_ast: &python_ast,
        };
        let semantic = SemanticModel::new(&[], Path::new(file), module);
        let mut checker = Checker::new(&stylist, semantic);
        checker.see_imports(&python_ast);

        for stmt in python_ast.iter() {
            if let ast::Stmt::ClassDef(class) = stmt {
                // we only care about class definitions
                class_mermaider(&mut class_diagram, &checker, class, 1);
            }
        }
    }

    class_diagram
}

fn parse_folder(path: &Path) -> std::io::Result<Vec<String>> {
    let mut parsed_files = vec![];

    let types = TypesBuilder::new()
        .add_defaults()
        .select("python")
        .build()
        .unwrap();

    for result in WalkBuilder::new(path).types(types).build() {
        match result {
            Ok(entry) => {
                if entry.path().is_dir() {
                    // we're only doing files here
                    continue;
                }

                if let Some(filename) = entry.path().to_str() {
                    parsed_files.push(filename.to_string());
                }
            }
            Err(err) => {
                error!("Error walking path: {:?}", err);
            }
        }
    }

    Ok(parsed_files)
}
