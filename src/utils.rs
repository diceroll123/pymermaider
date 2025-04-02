use ruff_python_ast::{Expr, Keyword};

use ruff_python_semantic::SemanticModel;

pub fn is_abc_class(bases: &[Expr], keywords: &[Keyword], semantic: &SemanticModel) -> bool {
    keywords.iter().any(|keyword| {
        keyword.arg.as_ref().is_some_and(|arg| arg == "metaclass")
            && semantic
                .resolve_qualified_name(&keyword.value)
                .is_some_and(|qualified_name| {
                    matches!(qualified_name.segments(), ["abc", "ABCMeta"])
                })
    }) || bases.iter().any(|base| {
        semantic
            .resolve_qualified_name(base)
            .is_some_and(|qualified_name| matches!(qualified_name.segments(), ["abc", "ABC"]))
    })
}
