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

/// Decide which brackets can be rendered as Mermaid's `~` generic delimiter.
///
/// `~` opens and closes alike, so a nested group is only unambiguous when it ends right where its
/// parent ends (`dict[str, list[int]]`). Groups followed by a sibling, and everything inside them,
/// must use entity codes instead. Unmatched brackets keep the plain `~` mapping.
fn tilde_brackets(bytes: &[u8]) -> Vec<bool> {
    let mut matching = vec![None; bytes.len()];
    let mut stack = Vec::new();
    for (i, &c) in bytes.iter().enumerate() {
        match c {
            b'[' => stack.push(i),
            b']' => {
                if let Some(open) = stack.pop() {
                    matching[open] = Some(i);
                    matching[i] = Some(open);
                }
            }
            _ => {}
        }
    }

    let mut tilde = vec![true; bytes.len()];
    // Open brackets still being walked, each with whether it already has a nested group.
    let mut parents: Vec<(usize, bool)> = Vec::new();
    for (i, &c) in bytes.iter().enumerate() {
        match (c, matching[i]) {
            (b'[', Some(close)) => {
                let ok = parents.last().is_none_or(|&(p, has_child)| {
                    !has_child && tilde[p] && matching[p] == Some(close + 1)
                });
                if let Some((_, has_child)) = parents.last_mut() {
                    *has_child = true;
                }
                tilde[i] = ok;
                tilde[close] = ok;
                parents.push((i, false));
            }
            (b']', Some(_)) => {
                parents.pop();
            }
            _ => {}
        }
    }
    tilde
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
        // Brackets are ASCII, so byte offsets from `char_indices` line up with the byte scan.
        // With at most one `[` there is nothing nested, so every bracket can be `~`.
        let tilde = if self.bytes().filter(|&b| b == b'[').count() > 1 {
            tilde_brackets(self.as_bytes())
        } else {
            Vec::new()
        };
        let use_tilde = |i: usize| tilde.get(i).copied().unwrap_or(true);
        let mut out = String::with_capacity(self.len());
        for (i, c) in self.char_indices() {
            match c {
                '[' | ']' if use_tilde(i) => out.push('~'),
                '[' => out.push_str("#91;"),
                ']' => out.push_str("#93;"),
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
    fn test_escape_type_nested_siblings_use_entities() {
        assert_eq!(
            "tuple[tuple[int, int], tuple[int, int]]".escape_type(),
            "tuple~tuple#91;int, int#93;, tuple#91;int, int#93;~"
        );
        assert_eq!(
            "tuple[tuple[int], int]".escape_type(),
            "tuple~tuple#91;int#93;, int~"
        );
    }

    #[test]
    fn test_escape_type_nested_chain() {
        assert_eq!(
            "list[list[list[int]]]".escape_type(),
            "list~list~list~int~~~"
        );
    }

    #[test]
    fn test_escape_type_unbalanced() {
        assert_eq!("list[int".escape_type(), "list~int");
        assert_eq!("int]".escape_type(), "int~");
    }

    #[test]
    fn test_escape_type_plain() {
        assert_eq!("int".escape_type(), "int");
    }
}
