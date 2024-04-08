mod class_diagram;

#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use crate::class_diagram::ClassDiagram;
use clap::{App, Arg};
use ignore::{types::TypesBuilder, WalkBuilder};
use itertools::Itertools;
use rustpython_parser::{
    ast::{self, Expr, Ranged},
    Parse,
};
use std::path::{Path, PathBuf};

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
            let parsed_file = parse_python_file(path);
            if let Err(e) = &parsed_file.result {
                println!("Error in file {:?}: {:?}", path, e);
            }
            let diagram = make_mermaid(vec![parsed_file]);
            let wrote_file = diagram.write_to_file(path.file_name().unwrap().to_str().unwrap());
            if wrote_file {
                written += 1;
            }
        } else if path.is_dir() {
            let parsed_files = parse_folder(path).unwrap();

            let multiple_files = matches.is_present("multiple");

            if multiple_files {
                for parsed_file in parsed_files.iter() {
                    let diagram = make_mermaid(vec![parsed_file.clone()]);
                    let wrote_file = diagram
                        .write_to_file(parsed_file.filename.file_name().unwrap().to_str().unwrap());
                    if wrote_file {
                        written += 1;
                    }
                }
            } else {
                let diagram = make_mermaid(parsed_files);

                let wrote_file = diagram.write_to_file(
                    path.canonicalize()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                );
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

fn grab_annotations(source: &str, annotation: &Expr) -> String {
    // not a great solution, but it works for now
    // I'd like for this to unparse the expr instead
    let range = annotation.range();
    source[range]
        .lines()
        .map(|line| line.trim_start())
        .join(" ")
}

fn stmt_mermaider(file: &ParsedFile, stmt: &ast::Stmt, indent_level: usize) -> String {
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

            let ast::Expr::Name(ast::ExprName { id: target, .. }) = target.as_ref() else {
                return res;
            };

            let target_name = target.to_string();

            let annotation_name = grab_annotations(file.source.as_ref(), annotation.as_ref());

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

        ast::Stmt::AsyncFunctionDef(ast::StmtAsyncFunctionDef {
            name,
            args,
            returns,
            ..
        })
        | ast::Stmt::FunctionDef(ast::StmtFunctionDef {
            name,
            args,
            returns,
            ..
        }) => {
            let is_async = matches!(stmt, ast::Stmt::AsyncFunctionDef(_));
            let function_name = name.to_string();

            let is_private = function_name.starts_with('_');

            let mut arg_list = Vec::new();

            if !args.as_ref().posonlyargs.is_empty() {
                arg_list.extend(
                    args.as_ref()
                        .posonlyargs
                        .iter()
                        .map(|arg| arg.def.arg.to_string()),
                );

                arg_list.push("/".to_string());
            }

            arg_list.extend(args.as_ref().args.iter().map(|arg| arg.def.arg.to_string()));

            if args.as_ref().vararg.is_some() {
                arg_list.push(format!("*{}", args.as_ref().vararg.as_ref().unwrap().arg));
            }

            if !args.as_ref().kwonlyargs.is_empty() {
                arg_list.push("*".to_string());
                arg_list.extend(
                    args.as_ref()
                        .kwonlyargs
                        .iter()
                        .map(|arg| arg.def.arg.to_string()),
                );
            }

            if args.as_ref().kwarg.is_some() {
                arg_list.push(format!("**{}", args.as_ref().kwarg.as_ref().unwrap().arg));
            }

            let returns = match returns {
                Some(target) => grab_annotations(file.source.as_ref(), target.as_ref()),
                None => "".to_string(),
            };

            let mut res = String::new();
            res.push_str(&TAB.repeat(indent_level));
            res.push_str(&format!(
                "{} {}{}({}) {}\n",
                if is_private { "-" } else { "+" },
                if is_async { "async " } else { "" },
                &function_name,
                arg_list.join(", "),
                returns,
            ));
            res
        }

        ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) => {
            let mut res = String::new();

            let value_type = match value.as_ref() {
                ast::Expr::BoolOp(_) => "bool",
                ast::Expr::BinOp(_) | ast::Expr::UnaryOp(_) => "int",
                ast::Expr::Lambda(_) => "Callable",
                ast::Expr::DictComp(_) | ast::Expr::Dict(_) => "dict",
                ast::Expr::Set(_) | ast::Expr::SetComp(_) => "set",
                ast::Expr::FormattedValue(_) | ast::Expr::JoinedStr(_) => "str",
                ast::Expr::Constant(_) => match value.as_ref() {
                    ast::Expr::Constant(ast::ExprConstant { value, .. }) => match value {
                        ast::Constant::None => "None",
                        ast::Constant::Bool(_) => "bool",
                        ast::Constant::Bytes(_) => "bytes",
                        ast::Constant::Int(_) => "int",
                        ast::Constant::Str(_) => "str",
                        ast::Constant::Ellipsis => "...",
                        ast::Constant::Float(_) => "float",
                        _ => "",
                    },
                    _ => "",
                },
                ast::Expr::ListComp(_) | ast::Expr::List(_) => "list",
                ast::Expr::Tuple(_) => "tuple",
                _ => "",
            };

            for target in targets {
                let ast::Expr::Name(ast::ExprName { id: target, .. }) = target else {
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
    file: &ParsedFile,
    class: &ast::StmtClassDef,
    indent_level: usize,
) {
    let class_name = class.name.to_string();
    let mut res = String::new();

    let use_tab = TAB.repeat(indent_level);

    res.push_str(&use_tab);
    res.push_str(&format!("class {} {{\n", &class_name));
    for stmt in class.body.iter() {
        res.push_str(&stmt_mermaider(file, stmt, indent_level + 1));
    }
    res.push_str(&use_tab);
    res.push('}');

    class_diagram.classes.push(res);

    for base in class.bases.iter() {
        let ast::Expr::Name(ast::ExprName { id: base, .. }) = base else {
            continue;
        };

        let base_name = base.to_string();

        let relationship = format!("{}{} --|> {}\n", use_tab, class_name, base_name);

        class_diagram.relationships.push(relationship);
    }
}

fn make_mermaid(parsed_files: Vec<ParsedFile>) -> ClassDiagram {
    let mut class_diagram = ClassDiagram::new();

    for file in parsed_files.iter() {
        if let Ok(stmts) = &file.result {
            for stmt in stmts.iter() {
                if let ast::Stmt::ClassDef(class) = stmt {
                    // we only care about class definitions
                    class_mermaider(&mut class_diagram, file, class, 1);
                }
            }
        }
    }

    class_diagram
}

fn parse_folder(path: &Path) -> std::io::Result<Vec<ParsedFile>> {
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

                let parsed_file = parse_python_file(entry.path());
                if let Err(y) = &parsed_file.result {
                    error!("Error in file {:?} {:?}", entry.path(), y);
                }
                parsed_files.push(parsed_file);
            }
            Err(err) => {
                error!("Error walking path: {:?}", err);
            }
        }
    }

    Ok(parsed_files)
}

fn parse_python_file(filename: &Path) -> ParsedFile {
    info!("Parsing file {:?}", filename);
    match std::fs::read_to_string(filename) {
        Err(e) => ParsedFile {
            filename: Box::new(filename.to_path_buf()),
            source: "".to_string(),
            result: Err(e.to_string()),
        },
        Ok(source) => {
            let result =
                ast::Suite::parse(&source, &filename.to_string_lossy()).map_err(|e| e.to_string());
            ParsedFile {
                filename: Box::new(filename.to_path_buf()),
                source,
                result,
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedFile {
    filename: Box<PathBuf>,
    source: String,
    result: ParseResult,
}

type ParseResult = Result<Vec<ast::Stmt>, String>;
