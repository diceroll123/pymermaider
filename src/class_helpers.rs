/// Helper traits and utilities for working with Python class definitions
use crate::ast;
use ruff_python_ast::name::QualifiedName;
use ruff_python_ast::Arguments;
use ruff_python_semantic::analyze::class::is_enumeration;
use ruff_python_semantic::analyze::visibility::is_final;
use ruff_python_semantic::SemanticModel;

/// Helper methods for qualified names in diagram context
pub trait QualifiedNameHelpers {
    fn normalize_name(&self) -> String;
}

impl QualifiedNameHelpers for QualifiedName<'_> {
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

/// Helper methods for class definitions
pub trait ClassDefHelpers {
    fn is_abstract(&self, semantic: &SemanticModel) -> bool;
    fn is_final(&self, semantic: &SemanticModel) -> bool;
    fn is_enum(&self, semantic: &SemanticModel) -> bool;
    fn is_protocol(&self, semantic: &SemanticModel) -> bool;
    fn is_dataclass(&self, semantic: &SemanticModel) -> bool;
}

impl ClassDefHelpers for ast::StmtClassDef {
    fn is_abstract(&self, semantic: &SemanticModel) -> bool {
        let Some(Arguments { args, keywords, .. }) = self.arguments.as_deref() else {
            return false;
        };

        for base in args.iter().chain(keywords.iter().map(|kw| &kw.value)) {
            if let Some(qualified_name) = semantic.resolve_qualified_name(base) {
                if matches!(
                    qualified_name.segments(),
                    ["abc", "ABC"] | ["abc", "ABCMeta"]
                ) {
                    return true;
                }
            }
        }
        false
    }

    fn is_final(&self, semantic: &SemanticModel) -> bool {
        is_final(&self.decorator_list, semantic)
    }

    fn is_enum(&self, semantic: &SemanticModel) -> bool {
        is_enumeration(self, semantic)
    }

    fn is_protocol(&self, semantic: &SemanticModel) -> bool {
        let Some(Arguments { args, keywords, .. }) = self.arguments.as_deref() else {
            return false;
        };

        if args.len() + keywords.len() != 1 {
            return false;
        }

        for base in args.iter().chain(keywords.iter().map(|kw| &kw.value)) {
            if let Some(qualified_name) = semantic.resolve_qualified_name(base) {
                if matches!(
                    qualified_name.segments(),
                    ["typing", "Protocol"] | ["typing_extensions", "Protocol"]
                ) {
                    return true;
                }
            }
        }
        false
    }

    fn is_dataclass(&self, semantic: &SemanticModel) -> bool {
        for decorator in &self.decorator_list {
            // Check the decorator expression directly (for @dataclass)
            if let Some(qualified_name) = semantic.resolve_qualified_name(&decorator.expression) {
                if matches!(
                    qualified_name.segments(),
                    ["dataclasses", "dataclass"] | ["pydantic", "dataclasses", "dataclass"]
                ) {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;
    use ruff_linter::source_kind::SourceKind;
    use ruff_linter::Locator;
    use ruff_python_ast::PySourceType;
    use ruff_python_codegen::Stylist;
    use ruff_python_parser::parse_unchecked_source;
    use ruff_python_semantic::{Module, ModuleKind, ModuleSource, SemanticModel};
    use std::path::Path;
    use std::path::PathBuf;

    fn find_class<'a>(python_ast: &'a [ast::Stmt], name: &str) -> &'a ast::StmtClassDef {
        python_ast
            .iter()
            .find_map(|stmt| match stmt {
                ast::Stmt::ClassDef(class) if class.name.as_str() == name => Some(class),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected {name} class"))
    }

    #[test]
    fn is_abstract_true_when_any_base_is_abc() {
        let source = r#"
from abc import ABC
class Mixin: ...
class Thing(ABC, Mixin): ...
"#;

        let file = PathBuf::from("test.py");
        let source_kind = SourceKind::Python(source.to_string());
        let locator = Locator::new(source_kind.source_code());
        let parsed = parse_unchecked_source(source_kind.source_code(), PySourceType::from(&file));
        let stylist = Stylist::from_tokens(parsed.tokens(), source_kind.source_code());
        let python_ast = parsed.into_suite();
        let module = Module {
            kind: ModuleKind::Module,
            source: ModuleSource::File(Path::new(&file)),
            python_ast: &python_ast,
            name: None,
        };
        let semantic = SemanticModel::new(&[], Path::new(&file), module);
        let mut checker = Checker::new(&stylist, &locator, semantic);
        checker.see_imports(&python_ast);

        let thing = find_class(&python_ast, "Thing");

        assert!(thing.is_abstract(checker.semantic()));
    }

    #[test]
    fn is_abstract_false_when_no_bases_are_abc() {
        let source = r#"
class A: ...
class B: ...
class Thing(A, B): ...
"#;

        let file = PathBuf::from("test.py");
        let source_kind = SourceKind::Python(source.to_string());
        let locator = Locator::new(source_kind.source_code());
        let parsed = parse_unchecked_source(source_kind.source_code(), PySourceType::from(&file));
        let stylist = Stylist::from_tokens(parsed.tokens(), source_kind.source_code());
        let python_ast = parsed.into_suite();
        let module = Module {
            kind: ModuleKind::Module,
            source: ModuleSource::File(Path::new(&file)),
            python_ast: &python_ast,
            name: None,
        };
        let semantic = SemanticModel::new(&[], Path::new(&file), module);
        let mut checker = Checker::new(&stylist, &locator, semantic);
        checker.see_imports(&python_ast);

        let thing = find_class(&python_ast, "Thing");

        assert!(!thing.is_abstract(checker.semantic()));
    }

    #[test]
    fn is_abstract_true_for_abc_abcmeta_metaclass() {
        let source = r#"
from abc import ABCMeta
class Mixin: ...
class Thing(Mixin, metaclass=ABCMeta): ...
"#;

        let file = PathBuf::from("test.py");
        let source_kind = SourceKind::Python(source.to_string());
        let locator = Locator::new(source_kind.source_code());
        let parsed = parse_unchecked_source(source_kind.source_code(), PySourceType::from(&file));
        let stylist = Stylist::from_tokens(parsed.tokens(), source_kind.source_code());
        let python_ast = parsed.into_suite();
        let module = Module {
            kind: ModuleKind::Module,
            source: ModuleSource::File(Path::new(&file)),
            python_ast: &python_ast,
            name: None,
        };
        let semantic = SemanticModel::new(&[], Path::new(&file), module);
        let mut checker = Checker::new(&stylist, &locator, semantic);
        checker.see_imports(&python_ast);

        let thing = find_class(&python_ast, "Thing");

        assert!(thing.is_abstract(checker.semantic()));
    }
}
