//! Glob pattern compilation utilities.

use crate::prelude::*;
use globset::GlobBuilder;

/// Compile a glob for argument matching, where `/` is not treated as a separator.
pub fn compile_arg_glob(pattern: &str, case_insensitive: bool) -> Option<GlobMatcher> {
    compile_glob(pattern, false, case_insensitive)
}

/// Compile a glob for path matching, where `/` acts as a literal separator.
pub fn compile_path_glob(pattern: &str) -> Option<GlobMatcher> {
    compile_glob(pattern, true, false)
}

fn compile_glob(
    pattern: &str,
    literal_separator: bool,
    case_insensitive: bool,
) -> Option<GlobMatcher> {
    if !is_glob(pattern) {
        return None;
    }
    match GlobBuilder::new(pattern)
        .literal_separator(literal_separator)
        .case_insensitive(case_insensitive)
        .build()
    {
        Ok(glob) => Some(glob.compile_matcher()),
        Err(e) => {
            error!("Failed to compile glob pattern: {}", e);
            None
        }
    }
}

fn is_glob(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('{') || pattern.contains('[')
}
