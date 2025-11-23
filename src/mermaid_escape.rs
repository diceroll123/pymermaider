/// Trait for escaping strings for Mermaid diagram rendering
pub trait MermaidEscape {
    /// Escape leading underscores for Mermaid diagrams.
    /// Mermaid interprets __ as formatting, so we escape leading underscores with backslashes.
    fn escape_underscores(&self) -> String;
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
            self.to_string()
        }
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
}
