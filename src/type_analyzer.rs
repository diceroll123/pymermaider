/// Type analysis utilities for extracting and analyzing Python types from AST
use crate::checker::Checker;
use ruff_python_ast::name::QualifiedName;
use ruff_python_ast::Expr;

// Built-in Python types that should not be treated as composition relationships
const BUILTIN_TYPES: &[&str] = &[
    "int", "str", "float", "bool", "bytes", "dict", "list", "tuple", "set", "None",
];

/// Extract type names from annotations for composition relationship detection.
/// Returns the type name if it represents a potential composition (non-builtin, non-typing class).
///
/// # Examples
/// - `foo: MyClass` → Some("MyClass")
/// - `foo: list[MyClass]` → Some("MyClass")
/// - `foo: Optional[MyClass]` → Some("MyClass")
/// - `foo: X | Y` → Some("X")
/// - `foo: int` → None (builtin)
pub fn extract_composition_type(annotation: &Expr, checker: &Checker) -> Option<String> {
    match annotation {
        // Simple name: foo: MyClass
        Expr::Name(name) => {
            let type_name = name.id.to_string();

            // Skip built-in types
            if BUILTIN_TYPES.contains(&type_name.as_str()) {
                return None;
            }

            // Try to resolve qualified name
            if let Some(qualified) = checker.semantic().resolve_qualified_name(annotation) {
                let segments = qualified.segments();
                // Skip built-in types and typing module types
                if matches!(segments[0], "builtins" | "typing" | "") {
                    return None;
                }
                Some(segments.join("."))
            } else {
                // If we can't resolve it, it might be a local class - return the name
                Some(type_name)
            }
        }
        // Subscript: foo: list[MyClass], Optional[MyClass], etc.
        Expr::Subscript(subscript) => {
            // Try to extract the inner type from containers
            match subscript.slice.as_ref() {
                Expr::Name(_) => extract_composition_type(subscript.slice.as_ref(), checker),
                Expr::Tuple(tuple) => {
                    // For Union, get first non-None type
                    for elt in &tuple.elts {
                        if let Some(type_name) = extract_composition_type(elt, checker) {
                            return Some(type_name);
                        }
                    }
                    None
                }
                _ => None,
            }
        }
        // Binary op for union types (X | Y)
        Expr::BinOp(binop) => {
            if let Some(left) = extract_composition_type(binop.left.as_ref(), checker) {
                return Some(left);
            }
            extract_composition_type(binop.right.as_ref(), checker)
        }
        _ => None,
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

/// Check if a qualified name represents a Generic base class
fn is_generic_base(name: &QualifiedName) -> bool {
    matches!(
        name.segments(),
        ["typing", "Generic"] | ["typing_extensions", "Generic"]
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
