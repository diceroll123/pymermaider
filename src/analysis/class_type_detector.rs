use super::checker::Checker;
use super::class_helpers::{is_abc_qualified_name, ClassDefHelpers};
/// Utilities for detecting and classifying Python class types
use crate::ast;
use crate::render::renderer::ClassType;
use ruff_python_semantic::SemanticModel;

/// Determines the type of a Python class based on its properties and decorators.
///
/// The precedence order is:
/// 1. Interface (Protocol)
/// 2. Dataclass
/// 3. Abstract
/// 4. Enumeration
/// 5. Final
/// 6. Regular
pub struct ClassTypeDetector<'a> {
    semantic: &'a SemanticModel<'a>,
}

impl<'a> ClassTypeDetector<'a> {
    #[must_use]
    pub const fn new(checker: &'a Checker) -> Self {
        Self {
            semantic: checker.semantic(),
        }
    }

    /// Determine the `ClassType` for a given class definition
    pub fn detect_type(&self, class: &ast::StmtClassDef) -> ClassType {
        if class.is_protocol(self.semantic) {
            ClassType::Interface
        } else if self.is_abstract(class) {
            // An abstract dataclass is still abstract: that is the more structural fact
            ClassType::Abstract
        } else if class.is_dataclass(self.semantic) {
            ClassType::Dataclass
        } else if class.is_named_tuple(self.semantic) {
            ClassType::NamedTuple
        } else if class.is_typed_dict(self.semantic) {
            ClassType::TypedDict
        } else if class.is_enum(self.semantic) {
            ClassType::Enumeration
        } else if class.is_final(self.semantic) {
            ClassType::Final
        } else {
            ClassType::Regular
        }
    }

    /// Check if a class is abstract.
    /// This includes classes that:
    /// - Have abstract methods (via decorators)
    /// - Inherit from ABC or `ABCMeta`
    fn is_abstract(&self, class: &ast::StmtClassDef) -> bool {
        // Check if class has abstract methods via trait
        if class.is_abstract(self.semantic) {
            return true;
        }

        // Check if any base class is ABC
        class.bases().iter().any(|base| {
            self.semantic
                .resolve_qualified_name(base)
                .is_some_and(|name| is_abc_qualified_name(&name))
        })
    }

    /// Check if a base class is abstract or a protocol.
    /// Used for determining relationship types (solid vs dotted lines).
    pub fn is_stdlib_abstract_or_protocol(&self, base_expr: &ast::Expr) -> bool {
        // Check if it's a standard library Protocol or ABC
        let base_expr = match base_expr {
            ast::Expr::Subscript(subscript) => subscript.value.as_ref(),
            other => other,
        };
        self.semantic
            .resolve_qualified_name(base_expr)
            .is_some_and(|name| {
                is_abc_qualified_name(&name)
                    || matches!(name.segments(), ["typing", "Protocol"])
                    || matches!(name.segments(), ["typing_extensions", "Protocol"])
            })
    }
}
