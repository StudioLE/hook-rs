//! Builder for [`OptionSchema`].

use crate::prelude::*;

/// Build an [`OptionSchema`] with sensible defaults.
///
/// Default: `value = None` (bool flag), `repeatable = false`.
pub struct OptionSchemaBuilder {
    names: Vec<String>,
    value: Option<ValueConstraint>,
    repeatable: bool,
}

impl OptionSchemaBuilder {
    /// Create a new builder with the given option names.
    pub fn new<T>(names: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<String>,
    {
        Self {
            names: names.into_iter().map(Into::into).collect(),
            value: None,
            repeatable: false,
        }
    }

    /// Set the value constraint.
    pub fn with_value(mut self, value: ValueConstraint) -> Self {
        self.value = Some(value);
        self
    }

    /// Mark this option as repeatable.
    pub fn with_repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }

    /// Build the [`OptionSchema`].
    pub fn build(self) -> OptionSchema {
        OptionSchema {
            names: self.names,
            value: self.value,
            repeatable: self.repeatable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_schema_builder_defaults() {
        // Act
        let output = OptionSchemaBuilder::new(["--verbose"]).build();

        // Assert
        assert_eq!(output.names, vec![String::from("--verbose")]);
        assert!(output.value.is_none());
        assert!(!output.repeatable);
    }

    #[test]
    fn option_schema_builder_multiple_names() {
        // Act
        let output = OptionSchemaBuilder::new(["-f", "--file"]).build();

        // Assert
        assert_eq!(
            output.names,
            vec![String::from("-f"), String::from("--file")]
        );
    }

    #[test]
    fn option_schema_builder_with_value() {
        // Act
        let output = OptionSchemaBuilder::new(["--color"])
            .with_value(ValueConstraint::any_of(vec![
                String::from("always"),
                String::from("never"),
                String::from("auto"),
            ]))
            .build();

        // Assert
        assert!(
            output
                .value
                .as_ref()
                .is_some_and(ValueConstraint::is_any_of)
        );
    }

    #[test]
    fn option_schema_builder_repeatable() {
        // Act
        let output = OptionSchemaBuilder::new(["-v"]).with_repeatable().build();

        // Assert
        assert!(output.repeatable);
    }
}
