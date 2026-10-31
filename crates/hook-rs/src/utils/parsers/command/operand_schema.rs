//! Operand definition within a [`CommandSchema`].

use crate::prelude::*;

/// Define a positional argument slot at one command level.
#[derive(Clone, Debug)]
pub struct OperandSchema {
    /// Documentation name for this operand.
    ///
    /// Example: `"service"`, `"path"`
    pub name: String,
    /// Constraint on the value this operand accepts.
    pub value: ValueConstraint,
    /// Whether this operand consumes all remaining arguments.
    ///
    /// Default: `false`
    pub variadic: bool,
    /// Whether this operand may be omitted.
    ///
    /// Optional operands MUST follow all required operands.
    /// Default: `false`
    pub optional: bool,
}
