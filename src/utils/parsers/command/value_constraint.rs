//! Value constraint for option and operand schemas.

use crate::prelude::*;

/// Constraint on the value an option or operand accepts.
///
/// - [`ValueConstraint::Any`] accepts all values
/// - [`ValueConstraint::Glob`] matches against a compiled glob pattern
/// - [`ValueConstraint::Regex`] matches against a compiled regex pattern
/// - [`ValueConstraint::AnyOf`] accepts exact matches from a set
#[derive(Clone, Debug)]
pub enum ValueConstraint {
    /// Unconstrained value.
    Any,
    /// Value must match a glob pattern.
    Glob {
        /// Source glob pattern string.
        pattern: String,
        /// Pre-compiled glob matcher.
        matcher: GlobMatcher,
    },
    /// Value must match a regex pattern.
    Regex {
        /// Source regex pattern string.
        pattern: String,
        /// Pre-compiled regex matcher.
        matcher: Regex,
    },
    /// Value must be one of these exact strings.
    AnyOf(Vec<String>),
}

impl ValueConstraint {
    /// Create a [`ValueConstraint::Glob`] from a pattern string.
    pub fn glob(pattern: impl Into<String>) -> Result<Self, Report<ValueConstraintError>> {
        let pattern = pattern.into();
        let matcher = Glob::new(&pattern)
            .change_context(ValueConstraintError::InvalidGlob)?
            .compile_matcher();
        Ok(Self::Glob { pattern, matcher })
    }

    /// Create a [`ValueConstraint::Regex`] from a pattern string.
    pub fn regex(pattern: impl Into<String>) -> Result<Self, Report<ValueConstraintError>> {
        let pattern = pattern.into();
        let matcher = Regex::new(&pattern).change_context(ValueConstraintError::InvalidRegex)?;
        Ok(Self::Regex { pattern, matcher })
    }

    /// Create a [`ValueConstraint::AnyOf`] from a set of valid strings.
    pub fn any_of(values: Vec<String>) -> Self {
        Self::AnyOf(values)
    }

    /// True if the given value satisfies this constraint.
    pub fn matches(&self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Glob { matcher, .. } => matcher.is_match(value),
            Self::Regex { matcher, .. } => matcher.is_match(value),
            Self::AnyOf(valid) => valid.iter().any(|v| v == value),
        }
    }

    /// True if this is [`ValueConstraint::Any`].
    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    /// True if this is [`ValueConstraint::Glob`].
    pub fn is_glob(&self) -> bool {
        matches!(self, Self::Glob { .. })
    }

    /// True if this is [`ValueConstraint::Regex`].
    pub fn is_regex(&self) -> bool {
        matches!(self, Self::Regex { .. })
    }

    /// True if this is [`ValueConstraint::AnyOf`].
    pub fn is_any_of(&self) -> bool {
        matches!(self, Self::AnyOf(_))
    }

    /// Source pattern string for display in error messages.
    ///
    /// - Returns the glob or regex pattern for pattern variants
    /// - Returns a comma-separated list for [`ValueConstraint::AnyOf`]
    /// - Returns `"any"` for [`ValueConstraint::Any`]
    pub fn description(&self) -> String {
        match self {
            Self::Any => String::from("any"),
            Self::Glob { pattern, .. } => format!("match glob: {pattern}"),
            Self::Regex { pattern, .. } => format!("match regex: {pattern}"),
            Self::AnyOf(valid) => format!("one of: {}", valid.join(", ")),
        }
    }
}

/// Errors returned by [`ValueConstraint`] constructors.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ValueConstraintError {
    /// Glob pattern failed to compile.
    #[error("Invalid glob pattern")]
    InvalidGlob,
    /// Regex pattern failed to compile.
    #[error("Invalid regex pattern")]
    InvalidRegex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_constraint_any_matches() {
        assert!(ValueConstraint::Any.matches("anything"));
    }

    #[test]
    fn value_constraint_any_accessor() {
        assert!(ValueConstraint::Any.is_any());
    }

    #[test]
    fn value_constraint_glob_matches() {
        // Arrange
        let constraint = ValueConstraint::glob("*.yml").expect("valid glob");

        // Assert
        assert!(constraint.matches("docker-compose.yml"));
    }

    #[test]
    fn value_constraint_glob_miss() {
        // Arrange
        let constraint = ValueConstraint::glob("*.yml").expect("valid glob");

        // Assert
        assert!(!constraint.matches("config.toml"));
    }

    #[test]
    fn value_constraint_glob_accessor() {
        // Arrange
        let constraint = ValueConstraint::glob("*.yml").expect("valid glob");

        // Assert
        assert!(constraint.is_glob());
    }

    #[test]
    fn value_constraint_regex_matches() {
        // Arrange
        let constraint = ValueConstraint::regex(r"^\d+$").expect("valid regex");

        // Assert
        assert!(constraint.matches("42"));
    }

    #[test]
    fn value_constraint_regex_miss() {
        // Arrange
        let constraint = ValueConstraint::regex(r"^\d+$").expect("valid regex");

        // Assert
        assert!(!constraint.matches("abc"));
    }

    #[test]
    fn value_constraint_regex_accessor() {
        // Arrange
        let constraint = ValueConstraint::regex(r"^\d+$").expect("valid regex");

        // Assert
        assert!(constraint.is_regex());
    }

    #[test]
    fn value_constraint_any_of_matches() {
        // Arrange
        let constraint = ValueConstraint::any_of(vec![String::from("start"), String::from("stop")]);

        // Assert
        assert!(constraint.matches("start"));
    }

    #[test]
    fn value_constraint_any_of_miss() {
        // Arrange
        let constraint = ValueConstraint::any_of(vec![String::from("start"), String::from("stop")]);

        // Assert
        assert!(!constraint.matches("explode"));
    }

    #[test]
    fn value_constraint_any_of_accessor() {
        // Arrange
        let constraint = ValueConstraint::any_of(vec![String::from("start"), String::from("stop")]);

        // Assert
        assert!(constraint.is_any_of());
    }

    #[test]
    fn value_constraint_glob_invalid() {
        // Act
        let error = ValueConstraint::glob("[invalid").expect_err("should fail");

        // Assert
        assert_eq!(*error.current_context(), ValueConstraintError::InvalidGlob);
    }

    #[test]
    fn value_constraint_regex_invalid() {
        // Act
        let error = ValueConstraint::regex("[invalid").expect_err("should fail");

        // Assert
        assert_eq!(*error.current_context(), ValueConstraintError::InvalidRegex);
    }
}
