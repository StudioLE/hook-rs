//! Builder for [`OperandSchema`].

use crate::prelude::*;

/// Build an [`OperandSchema`] with sensible defaults.
///
/// Default: `value = ValueConstraint::Any`, `variadic = false`, `optional = false`.
pub struct OperandSchemaBuilder {
    name: String,
    value: ValueConstraint,
    variadic: bool,
    optional: bool,
}

impl OperandSchemaBuilder {
    /// Create a new builder with the given operand name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: ValueConstraint::Any,
            variadic: false,
            optional: false,
        }
    }

    /// Set the value constraint.
    pub fn with_value(mut self, value: ValueConstraint) -> Self {
        self.value = value;
        self
    }

    /// Mark this operand as variadic.
    pub fn with_variadic(mut self) -> Self {
        self.variadic = true;
        self
    }

    /// Mark this operand as optional.
    pub fn with_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Build the [`OperandSchema`].
    pub fn build(self) -> OperandSchema {
        OperandSchema {
            name: self.name,
            value: self.value,
            variadic: self.variadic,
            optional: self.optional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operand_schema_builder_defaults() {
        // Act
        let output = OperandSchemaBuilder::new("service").build();

        // Assert
        assert_eq!(output.name, "service");
        assert!(output.value.is_any());
        assert!(!output.variadic);
        assert!(!output.optional);
    }

    #[test]
    fn operand_schema_builder_with_value() {
        // Act
        let output = OperandSchemaBuilder::new("action")
            .with_value(ValueConstraint::any_of(vec![
                String::from("start"),
                String::from("stop"),
            ]))
            .build();

        // Assert
        assert!(output.value.is_any_of());
    }

    #[test]
    fn operand_schema_builder_variadic() {
        // Act
        let output = OperandSchemaBuilder::new("args").with_variadic().build();

        // Assert
        assert!(output.variadic);
    }

    #[test]
    fn operand_schema_builder_optional() {
        // Act
        let output = OperandSchemaBuilder::new("commit-ish")
            .with_optional()
            .build();

        // Assert
        assert!(output.optional);
    }
}
