/// Type analysis utilities for extracting and analyzing Python types from AST
use super::checker::Checker;
use ruff_python_ast::name::QualifiedName;
use ruff_python_ast::Expr;

// Built-in Python types that should not be treated as composition relationships
const BUILTIN_TYPES: &[&str] = &[
    "int", "str", "float", "bool", "bytes", "dict", "list", "tuple", "set", "None",
];

/// Extract type names from annotations for composition relationship detection.
/// Returns `(type_name, is_aggregation)` pairs. `is_aggregation = true` when the
/// type is wrapped in a collection or Optional, implying a weak (o--) relationship.
///
/// # Examples
/// - `foo: MyClass` → `[("MyClass", false)]` - composition
/// - `foo: list[MyClass]` → `[("MyClass", true)]` - aggregation
/// - `foo: Optional[MyClass]` → `[("MyClass", true)]` - aggregation
/// - `foo: MyClass | None` → `[("MyClass", true)]` - aggregation
/// - `foo: X | Y` → `[("X", false), ("Y", false)]` - composition
/// - `foo: "MyClass"` → `[("MyClass", false)]` - composition
/// - `foo: int` → `[]` (builtin)
pub fn extract_composition_types(annotation: &Expr, checker: &Checker) -> Vec<(String, bool)> {
    extract_inner(annotation, checker, false)
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

fn is_aggregation_container(subscript_value: &Expr, checker: &Checker) -> bool {
    // Resolve imported names (handles typing.Optional, typing.List, etc.)
    if checker
        .semantic()
        .resolve_qualified_name(subscript_value)
        .is_some_and(|qn| {
            matches!(
                qn.segments(),
                ["builtins", "list" | "dict" | "set" | "tuple" | "frozenset"]
                    | [
                        "typing" | "typing_extensions",
                        "Optional"
                            | "List"
                            | "Dict"
                            | "Set"
                            | "Tuple"
                            | "FrozenSet"
                            | "Sequence"
                            | "Iterable"
                            | "Iterator"
                    ]
            )
        })
    {
        return true;
    }
    // Bare builtin names: list[X], dict[K, V], set[X], etc. (Python 3.9+)
    matches!(
        subscript_value,
        Expr::Name(n) if matches!(
            n.id.as_str(),
            "list" | "dict" | "set" | "tuple" | "frozenset"
        )
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypingWrapper {
    /// `Literal[...]` holds values rather than types
    Literal,
    /// `Annotated[X, metadata...]` only has a type in its first element
    Annotated,
}

fn typing_wrapper(subscript_value: &Expr, checker: &Checker) -> Option<TypingWrapper> {
    let from_name = |name: &str| match name {
        "Literal" => Some(TypingWrapper::Literal),
        "Annotated" => Some(TypingWrapper::Annotated),
        _ => None,
    };
    match checker.semantic().resolve_qualified_name(subscript_value) {
        Some(qn) => match qn.segments() {
            ["typing" | "typing_extensions", name] => from_name(name),
            _ => None,
        },
        None => match subscript_value {
            Expr::Name(n) => from_name(n.id.as_str()),
            _ => None,
        },
    }
}

fn is_none_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::NoneLiteral(_)) || matches!(expr, Expr::Name(n) if n.id.as_str() == "None")
}

fn extract_inner(
    annotation: &Expr,
    checker: &Checker,
    is_aggregation: bool,
) -> Vec<(String, bool)> {
    match annotation {
        Expr::Name(name) => is_eligible_name(name.id.as_ref(), annotation, checker)
            .map(|n| vec![(n, is_aggregation)])
            .unwrap_or_default(),

        Expr::Subscript(subscript) => {
            let wrapper = typing_wrapper(&subscript.value, checker);
            // Literal["x"] holds values, not types.
            if wrapper == Some(TypingWrapper::Literal) {
                return vec![];
            }
            let agg = is_aggregation || is_aggregation_container(&subscript.value, checker);
            match subscript.slice.as_ref() {
                Expr::Name(_)
                | Expr::BinOp(_)
                | Expr::StringLiteral(_)
                | Expr::Subscript(_)
                | Expr::List(_) => extract_inner(subscript.slice.as_ref(), checker, agg),
                // Annotated[X, metadata...]: only the first element is a type.
                Expr::Tuple(tuple) if wrapper == Some(TypingWrapper::Annotated) => tuple
                    .elts
                    .first()
                    .map(|elt| extract_inner(elt, checker, agg))
                    .unwrap_or_default(),
                Expr::Tuple(tuple) => tuple
                    .elts
                    .iter()
                    .flat_map(|elt| extract_inner(elt, checker, agg))
                    .collect(),
                _ => vec![],
            }
        }

        // Callable[[A, B], R]: the parameter list
        Expr::List(list) => list
            .elts
            .iter()
            .flat_map(|elt| extract_inner(elt, checker, is_aggregation))
            .collect(),

        Expr::BinOp(binop) => {
            let optional_union =
                is_none_expr(binop.left.as_ref()) || is_none_expr(binop.right.as_ref());
            let agg = is_aggregation || optional_union;
            let mut out = extract_inner(binop.left.as_ref(), checker, agg);
            out.extend(extract_inner(binop.right.as_ref(), checker, agg));
            out
        }

        // Forward reference: foo: "MyClass" or foo: "Optional[A | B]"
        Expr::StringLiteral(literal) => {
            ruff_python_parser::parse_expression(literal.value.to_str())
                .map(|parsed| extract_inner(parsed.expr(), checker, is_aggregation))
                .unwrap_or_default()
        }

        _ => vec![],
    }
}

/// Extract generic type parameters from a base class expression.
/// Returns the type parameter(s) if the base is Generic[T] or similar.
///
/// # Examples
/// - `Generic[T]` → Some("T")
/// - `Generic[T, U]` → Some("T, U")
/// - `SomeClass` → None
pub fn extract_generic_params(base: &Expr, checker: &Checker) -> Option<String> {
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

    // Get the type string and extract just the parameter without Generic[]
    let type_var = checker.locator().to_source_code().slice(base);

    let start_idx = type_var.find('[').map(|idx| idx + 1)?;
    let end_idx = type_var.rfind(']')?;

    if start_idx < end_idx {
        Some(type_var[start_idx..end_idx].trim().to_owned())
    } else {
        None
    }
}

/// Check if a qualified name represents a Generic or Protocol base class
/// (both carry type parameters when subscripted)
fn is_generic_base(name: &QualifiedName) -> bool {
    matches!(
        name.segments(),
        ["typing" | "typing_extensions", "Generic" | "Protocol"]
    )
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
