use crate::ast;
use ast::{helpers::collect_import_from_member, identifier::Identifier, name::QualifiedName};
use itertools::Itertools;
use ruff_linter::Locator;
use ruff_python_ast::str::Quote;
use ruff_python_codegen::{Generator, Stylist};
use ruff_python_semantic::{
    BindingFlags, BindingId, BindingKind, FromImport, Import, SemanticModel, StarImport,
    SubmoduleImport,
};
use ruff_python_stdlib::builtins::{python_builtins, MAGIC_GLOBALS};
use ruff_text_size::TextRange;

/// Slimmed down version of the `Checker` struct from the `ruff_python_semantic` crate.
pub struct Checker<'a> {
    stylist: &'a Stylist<'a>,
    locator: &'a Locator<'a>,
    semantic: SemanticModel<'a>,
}

impl<'a> Checker<'a> {
    pub fn new(
        stylist: &'a Stylist<'a>,
        locator: &'a Locator<'a>,
        semantic: SemanticModel<'a>,
    ) -> Self {
        let mut checker = Self {
            stylist,
            locator,
            semantic,
        };
        checker.bind_builtins();
        checker
    }

    fn bind_builtins(&mut self) {
        for builtin in python_builtins(u8::MAX, false).chain(MAGIC_GLOBALS.iter().copied()) {
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

    pub fn locator(&self) -> &Locator<'a> {
        self.locator
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

    fn add_binding(
        &mut self,
        name: &'a str,
        range: TextRange,
        kind: BindingKind<'a>,
        flags: BindingFlags,
    ) -> BindingId {
        // Determine the scope to which the binding belongs.
        // Per [PEP 572](https://peps.python.org/pep-0572/#scope-of-the-target), named
        // expressions in generators and comprehensions bind to the scope that contains the
        // outermost comprehension.
        let scope_id = if kind.is_named_expr_assignment() {
            self.semantic
                .scopes
                .ancestor_ids(self.semantic.scope_id)
                .find_or_last(|scope_id| !self.semantic.scopes[*scope_id].kind.is_generator())
                .unwrap_or(self.semantic.scope_id)
        } else {
            self.semantic.scope_id
        };

        // Create the `Binding`.
        let binding_id = self.semantic.push_binding(range, kind, flags);

        // If the name is private, mark is as such.
        if name.starts_with('_') {
            self.semantic.bindings[binding_id].flags |= BindingFlags::PRIVATE_DECLARATION;
        }

        // If there's an existing binding in this scope, copy its references.
        if let Some(shadowed_id) = self.semantic.scopes[scope_id].get(name) {
            // If this is an annotation, and we already have an existing value in the same scope,
            // don't treat it as an assignment, but track it as a delayed annotation.
            if self.semantic.binding(binding_id).kind.is_annotation() {
                self.semantic
                    .add_delayed_annotation(shadowed_id, binding_id);
                return binding_id;
            }

            // Avoid shadowing builtins.
            let shadowed = &self.semantic.bindings[shadowed_id];
            if !matches!(
                shadowed.kind,
                BindingKind::Builtin | BindingKind::Deletion | BindingKind::UnboundException(_)
            ) {
                let references = shadowed.references.clone();
                let is_global = shadowed.is_global();
                let is_nonlocal = shadowed.is_nonlocal();

                // If the shadowed binding was global, then this one is too.
                if is_global {
                    self.semantic.bindings[binding_id].flags |= BindingFlags::GLOBAL;
                }

                // If the shadowed binding was non-local, then this one is too.
                if is_nonlocal {
                    self.semantic.bindings[binding_id].flags |= BindingFlags::NONLOCAL;
                }

                self.semantic.bindings[binding_id].references = references;
            }
        } else if let Some(shadowed_id) = self
            .semantic
            .scopes
            .ancestors(scope_id)
            .skip(1)
            .filter(|scope| scope.kind.is_function() || scope.kind.is_module())
            .find_map(|scope| scope.get(name))
        {
            // Otherwise, if there's an existing binding in a parent scope, mark it as shadowed.
            self.semantic
                .shadowed_bindings
                .insert(binding_id, shadowed_id);
        }

        // Add the binding to the scope.
        let scope = &mut self.semantic.scopes[scope_id];
        scope.add(name, binding_id);

        binding_id
    }

    pub fn see_imports(&mut self, stmts: &'a [ast::Stmt]) {
        for stmt in stmts {
            match stmt {
                ast::Stmt::Import(ast::StmtImport { names, .. }) => {
                    for alias in names {
                        let module = alias.name.split('.').next().unwrap();

                        self.semantic.add_module(module);

                        if alias.asname.is_none() && alias.name.contains('.') {
                            let qualified_name = QualifiedName::user_defined(&alias.name);
                            self.add_binding(
                                module,
                                alias.identifier(),
                                BindingKind::SubmoduleImport(SubmoduleImport {
                                    qualified_name: Box::new(qualified_name),
                                }),
                                BindingFlags::EXTERNAL,
                            );
                        } else {
                            let mut flags = BindingFlags::EXTERNAL;
                            if alias.asname.is_some() {
                                flags |= BindingFlags::ALIAS;
                            }
                            if alias
                                .asname
                                .as_ref()
                                .is_some_and(|asname| asname.as_str() == alias.name.as_str())
                            {
                                flags |= BindingFlags::EXPLICIT_EXPORT;
                            }

                            let name = alias.asname.as_ref().unwrap_or(&alias.name);
                            let qualified_name = QualifiedName::user_defined(&alias.name);
                            self.add_binding(
                                name,
                                alias.identifier(),
                                BindingKind::Import(Import {
                                    qualified_name: Box::new(qualified_name),
                                }),
                                flags,
                            );
                        }
                    }
                }
                ast::Stmt::ImportFrom(ast::StmtImportFrom {
                    names,
                    module,
                    level,
                    ..
                }) => {
                    let module = module.as_deref();
                    let level = *level;

                    // Mark the top-level module as "seen" by the semantic model.
                    if level == 0 {
                        if let Some(module) = module.and_then(|module| module.split('.').next()) {
                            self.semantic.add_module(module);
                        }
                    }

                    for alias in names {
                        if let Some("__future__") = module {
                            let name = alias.asname.as_ref().unwrap_or(&alias.name);
                            self.add_binding(
                                name,
                                alias.identifier(),
                                BindingKind::FutureImport,
                                BindingFlags::empty(),
                            );
                        } else if &alias.name == "*" {
                            self.semantic
                                .current_scope_mut()
                                .add_star_import(StarImport { level, module });
                        } else {
                            let mut flags = BindingFlags::EXTERNAL;
                            if alias.asname.is_some() {
                                flags |= BindingFlags::ALIAS;
                            }
                            if alias
                                .asname
                                .as_ref()
                                .is_some_and(|asname| asname.as_str() == alias.name.as_str())
                            {
                                flags |= BindingFlags::EXPLICIT_EXPORT;
                            }

                            // Given `from foo import bar`, `name` would be "bar" and `qualified_name` would
                            // be "foo.bar". Given `from foo import bar as baz`, `name` would be "baz"
                            // and `qualified_name` would be "foo.bar".
                            let name = alias.asname.as_ref().unwrap_or(&alias.name);

                            // Attempt to resolve any relative imports; but if we don't know the current
                            // module path, or the relative import extends beyond the package root,
                            // fallback to a literal representation (e.g., `[".", "foo"]`).
                            let qualified_name =
                                collect_import_from_member(level, module, &alias.name);
                            self.add_binding(
                                name,
                                alias.identifier(),
                                BindingKind::FromImport(FromImport {
                                    qualified_name: Box::new(qualified_name),
                                }),
                                flags,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
