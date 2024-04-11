use crate::ast;
use ruff_python_ast::str::Quote;
use ruff_python_codegen::{Generator, Stylist};
use ruff_python_semantic::SemanticModel;
use ruff_python_stdlib::builtins::{MAGIC_GLOBALS, PYTHON_BUILTINS};

pub struct Checker<'a> {
    stylist: &'a Stylist<'a>,
    semantic: SemanticModel<'a>,
}

impl<'a> Checker<'a> {
    pub fn new(stylist: &'a Stylist<'a>, semantic: SemanticModel<'a>) -> Self {
        let mut checker = Self { stylist, semantic };
        checker.bind_builtins();
        checker
    }

    fn bind_builtins(&mut self) {
        for builtin in PYTHON_BUILTINS.iter().chain(MAGIC_GLOBALS.iter()).copied() {
            // Add the builtin to the scope.
            let binding_id = self.semantic.push_builtin();
            let scope = self.semantic.global_scope_mut();
            scope.add(builtin, binding_id);
        }
    }

    pub fn generator(&self) -> Generator {
        Generator::new(
            self.stylist.indentation(),
            self.f_string_quote_style().unwrap_or(self.stylist.quote()),
            self.stylist.line_ending(),
        )
    }

    pub fn semantic(&self) -> &SemanticModel<'a> {
        &self.semantic
    }

    fn f_string_quote_style(&self) -> Option<Quote> {
        if !self.semantic.in_f_string() {
            return None;
        }

        // Find the quote character used to start the containing f-string.
        let ast::ExprFString { value, .. } = self
            .semantic
            .current_expressions()
            .find_map(|expr| expr.as_f_string_expr())?;
        Some(value.iter().next()?.quote_style().opposite())
    }
}
