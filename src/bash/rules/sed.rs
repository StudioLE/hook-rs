//! Rules for `sed` operations: allow read-only, deny in-place editing.

use crate::prelude::*;

/// Deny in-place `sed`, allow read-only `sed`.
pub fn sed_rules() -> Vec<BashRule> {
    vec![sed__in_place(), sed__read_only()]
}

/// Allow `sed` without `-i`/`--in-place`.
fn sed__read_only() -> BashRule {
    BashRule {
        id: "sed__read_only".to_owned(),
        command: "sed".to_owned(),
        without_any: Some(vec![ArgMatcher::new("-i"), ArgMatcher::new("--in-place")]),
        outcome: Outcome::allow("Read-only `sed`"),
        ..Default::default()
    }
}

/// Deny `sed -i`/`sed --in-place`; point at built-in `Edit`/`Write`.
fn sed__in_place() -> BashRule {
    BashRule {
        id: "sed__in_place".to_owned(),
        command: "sed".to_owned(),
        with_any: Some(vec![ArgMatcher::new("-i"), ArgMatcher::new("--in-place")]),
        outcome: Outcome::deny(
            "`sed -i` is blocked. Use the built-in `Edit` or `Write` tool instead",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn sed_in_place_short() {
        let outcome = evaluate_expect_outcome("sed -i 's/foo/bar/' file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sed_in_place_long() {
        let outcome = evaluate_expect_outcome("sed --in-place 's/foo/bar/' file.txt");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sed_read_only() {
        let outcome = evaluate_expect_outcome("sed -n '1,10p' file.txt");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
