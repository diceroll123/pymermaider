/// Utilities for detecting and classifying Python class types
use crate::ast;
use crate::checker::Checker;
use crate::class_helpers::ClassDefHelpers;
use crate::renderer::ClassType;
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
    pub fn new(checker: &'a Checker) -> Self {
        Self {
            semantic: checker.semantic(),
        }
    }

    /// Determine the ClassType for a given class definition
    pub fn detect_type(&self, class: &ast::StmtClassDef) -> ClassType {
        if class.is_protocol(self.semantic) {
            ClassType::Interface
        } else if class.is_dataclass(self.semantic) {
            ClassType::Dataclass
        } else if self.is_abstract(class) {
            ClassType::Abstract
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
    /// - Inherit from ABC or ABCMeta
    fn is_abstract(&self, class: &ast::StmtClassDef) -> bool {
        // Check if class has abstract methods via trait
        if class.is_abstract(self.semantic) {
            return true;
        }

        // Check if any base class is ABC
        class.bases().iter().any(|base| {
            self.semantic
                .resolve_qualified_name(base)
                .is_some_and(|name| {
                    matches!(
                        name.segments(),
                        ["abc", "ABC" | "ABCMeta"] | ["typing", "ABC"]
                    )
                })
        })
    }

    /// Check if a base class is abstract or a protocol.
    /// Used for determining relationship types (solid vs dotted lines).
    pub fn is_base_abstract_or_protocol(
        &self,
        base_name: &str,
        base_expr: &ast::Expr,
        protocol_classes: &std::collections::HashSet<String>,
        abstract_classes: &std::collections::HashSet<String>,
    ) -> bool {
        // Check if it's in our tracked sets (user-defined classes)
        if protocol_classes.contains(base_name) || abstract_classes.contains(base_name) {
            return true;
        }

        // Check if it's a standard library Protocol or ABC
        self.semantic
            .resolve_qualified_name(base_expr)
            .is_some_and(|name| {
                matches!(name.segments(), ["abc", "ABC" | "ABCMeta"])
                    || matches!(name.segments(), ["typing", "Protocol"])
                    || matches!(name.segments(), ["typing_extensions", "Protocol"])
            })
    }
}
