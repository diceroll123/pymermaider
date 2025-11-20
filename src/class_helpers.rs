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

        if args.len() + keywords.len() != 1 {
            return false;
        }

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
