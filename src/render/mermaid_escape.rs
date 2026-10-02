/// Trait for escaping strings for Mermaid diagram rendering
pub trait MermaidEscape {
    /// Escape leading underscores for Mermaid diagrams.
    /// Mermaid interprets __ as formatting, so we escape leading underscores with backslashes.
    fn escape_underscores(&self) -> String;

    /// Escape a type annotation or parameter list for use inside a class body.
    /// Mermaid uses `~` for generics and treats braces and quotes as syntax,
    /// so `[`/`]` become `~` and `{`, `}`, `"` become entity codes.
    fn escape_type(&self) -> String;
}

impl MermaidEscape for str {
    fn escape_underscores(&self) -> String {
        let leading_underscores = self.chars().take_while(|&c| c == '_').count();
        if leading_underscores > 0 {
            format!(
                "{}{}",
                r"\_".repeat(leading_underscores),
                &self[leading_underscores..]
            )
        } else {
            self.to_owned()
        }
    }

    fn escape_type(&self) -> String {
        let mut out = String::with_capacity(self.len());
        for c in self.chars() {
            match c {
                '[' | ']' => out.push('~'),
                '{' => out.push_str("#123;"),
                '}' => out.push_str("#125;"),
                '"' => out.push_str("#quot;"),
                _ => out.push(c),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_underscores() {
        assert_eq!("hello".escape_underscores(), "hello");
    }

    #[test]
    fn test_single_leading_underscore() {
        assert_eq!("_private".escape_underscores(), r"\_private");
    }

    #[test]
    fn test_double_leading_underscore() {
        assert_eq!("__init__".escape_underscores(), r"\_\_init__");
    }

    #[test]
    fn test_triple_leading_underscore() {
        assert_eq!("___triple".escape_underscores(), r"\_\_\_triple");
    }

    #[test]
    fn test_trailing_underscores_not_escaped() {
        assert_eq!("_method_".escape_underscores(), r"\_method_");
    }

    #[test]
    fn test_escape_type_generics() {
        assert_eq!("dict[str, list[int]]".escape_type(), "dict~str, list~int~~");
    }

    #[test]
    fn test_escape_type_braces_and_quotes() {
        assert_eq!(
            "Literal[\"a\"] | {}".escape_type(),
            "Literal~#quot;a#quot;~ | #123;#125;"
        );
    }

    #[test]
    fn test_escape_type_plain() {
        assert_eq!("int".escape_type(), "int");
    }
}
