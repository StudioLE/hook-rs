//! Logical connector between pipeline items.

use crate::prelude::*;

/// Logical connector between pipeline items.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Connector {
    And,
    Or,
    Semi,
}
