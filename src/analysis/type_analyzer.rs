/// Type analysis utilities for extracting and analyzing Python types from AST
use super::checker::Checker;
use crate::render::renderer::CompositionKind;
use ruff_python_ast::name::QualifiedName;
use ruff_python_ast::Expr;

// Built-in Python types that should not be treated as composition relationships
const BUILTIN_TYPES: &[&str] = &[
    "int", "str", "float", "bool", "bytes", "dict", "list", "tuple", "set", "None",
];

/// Extract type names from annotations for composition relationship detection.
/// Returns `(type_name, CompositionKind)` pairs:
/// - `Composition` for bare type references (`foo: MyClass`)
/// - `Optional` for optional references (`foo: Optional[MyClass]`, `foo: MyClass | None`)
/// - `Collection` for collection containers (`foo: list[MyClass]`, `foo: Sequence[MyClass]`)
pub fn extract_composition_types(
    annotation: &Expr,
    checker: &Checker,
) -> Vec<(String, CompositionKind)> {
    extract_inner(annotation, checker, CompositionKind::Composition)
}

fn is_eligible_name(type_name: &str, annotation: &Expr, checker: &Checker) -> Option<String> {
    if BUILTIN_TYPES.contains(&type_name) {
        return None;
    }
    if let Some(qualified) = checker.semantic().resolve_qualified_name(annotation) {
        let segments = qualified.segments();
        if matches!(segments[0], "builtins" | "typing" | "") {
            return None;
        }
        Some(segments.join("."))
    } else {
        Some(type_name.to_string())
    }
}

/// Returns the `CompositionKind` implied by a subscript container (e.g. `list` in `list[X]`),
/// or `None` if the expression is not a recognized container.
fn container_kind(subscript_value: &Expr, checker: &Checker) -> Option<CompositionKind> {
    if let Some(qn) = checker.semantic().resolve_qualified_name(subscript_value) {
        let segs = qn.segments();
        if matches!(segs, ["typing" | "typing_extensions", "Optional"]) {
            return Some(CompositionKind::Optional);
        }
        if matches!(
            segs,
            ["builtins", "list" | "dict" | "set" | "tuple" | "frozenset"]
                | [
                    "typing" | "typing_extensions",
                    "List"
                        | "Dict"
                        | "Set"
                        | "Tuple"
                        | "FrozenSet"
                        | "Sequence"
                        | "Iterable"
                        | "Iterator"
                        | "Collection"
                ]
        ) {
            return Some(CompositionKind::Collection);
        }
    }
    // Bare builtin names without import (Python 3.9+ generics like `list[X]`)
    if let Expr::Name(n) = subscript_value {
        if matches!(
            n.id.as_str(),
            "list" | "dict" | "set" | "tuple" | "frozenset"
        ) {
            return Some(CompositionKind::Collection);
        }
    }
    None
}

fn is_none_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::NoneLiteral(_)) || matches!(expr, Expr::Name(n) if n.id.as_str() == "None")
}

fn extract_inner(
    annotation: &Expr,
    checker: &Checker,
    kind: CompositionKind,
) -> Vec<(String, CompositionKind)> {
    match annotation {
        Expr::Name(name) => is_eligible_name(name.id.as_ref(), annotation, checker)
            .map(|n| vec![(n, kind)])
            .unwrap_or_default(),

        Expr::Subscript(subscript) => {
            let new_kind = container_kind(&subscript.value, checker).unwrap_or(kind);
            match subscript.slice.as_ref() {
                Expr::Name(_) => extract_inner(subscript.slice.as_ref(), checker, new_kind),
                Expr::Tuple(tuple) => {
                    // Union[X, None] or Union[X, Y] -- check if any element is None
                    let has_none = tuple.elts.iter().any(is_none_expr);
                    let tuple_kind = if has_none {
                        CompositionKind::Optional
                    } else {
                        new_kind
                    };
                    tuple
                        .elts
                        .iter()
                        .filter(|e| !is_none_expr(e))
                        .flat_map(|elt| extract_inner(elt, checker, tuple_kind))
                        .collect()
                }
                _ => vec![],
            }
        }

        // Binary union types: X | Y or X | None
        Expr::BinOp(binop) => {
            let has_none = is_none_expr(binop.left.as_ref()) || is_none_expr(binop.right.as_ref());
            let new_kind = if has_none {
                CompositionKind::Optional
            } else {
                kind
            };
            let mut out = extract_inner(binop.left.as_ref(), checker, new_kind);
            out.extend(extract_inner(binop.right.as_ref(), checker, new_kind));
            out
        }

        _ => vec![],
    }
}

/// Extract generic type parameters from a base class expression.
/// Returns the type parameter(s) if the base is Generic[T] or similar.
///
/// # Examples
/// - `Generic[T]` -> Some("T")
/// - `Generic[T, U]` -> Some("T, U")
/// - `SomeClass` -> None
pub fn extract_generic_params(base: &Expr, checker: &Checker) -> Option<String> {
    let Expr::Subscript(subscript) = base else {
        return None;
    };

    let base_name = checker
        .semantic()
        .resolve_qualified_name(&subscript.value)?;

    if !is_generic_base(&base_name) {
        return None;
    }

    let type_var = checker.locator().slice(base);

    let start_idx = type_var.find('[').map(|idx| idx + 1)?;
    let end_idx = type_var.rfind(']')?;

    if start_idx < end_idx {
        Some(type_var[start_idx..end_idx].trim().to_owned())
    } else {
        None
    }
}

fn is_generic_base(name: &QualifiedName) -> bool {
    matches!(name.segments(), ["typing" | "typing_extensions", "Generic"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types_constant() {
        assert!(BUILTIN_TYPES.contains(&"int"));
        assert!(BUILTIN_TYPES.contains(&"str"));
        assert!(!BUILTIN_TYPES.contains(&"MyClass"));
    }
}
