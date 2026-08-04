//! Rules for `rg` operations: allow read-only, deny short `-r` replacement.

use crate::prelude::*;

/// Deny `rg -r`, allow read-only `rg`.
pub fn rg_rules() -> Vec<BashRule> {
    vec![rg__replace(), rg__read_only()]
}

/// Allow `rg` without a short `-r`.
fn rg__read_only() -> BashRule {
    BashRule {
        id: "rg__read_only".to_owned(),
        command: "rg".to_owned(),
        without_any: Some(vec![ArgMatcher::new("-r")]),
        outcome: Outcome::allow("Read-only `rg`"),
        ..Default::default()
    }
}

/// Deny short `rg -r`; point at `rg -n` and the long `--replace` form.
///
/// `-r` is `--replace` in `rg`, not `--recursive`, so `-rn` silently replaces
/// every match with the literal `n`.
fn rg__replace() -> BashRule {
    BashRule {
        id: "rg__replace".to_owned(),
        command: "rg".to_owned(),
        with_any: Some(vec![ArgMatcher::new("-r")]),
        outcome: Outcome::deny(
            "`rg -r` is `--replace`, not `--recursive`, so it rewrites matches in the output. `rg` already recurses. Use `rg -n`, or `rg --replace=TEXT` to replace deliberately",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn rg_replace_concatenated() {
        let result = eval_rules(rg_rules(), "rg -rn 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_replace_separate() {
        let result = eval_rules(rg_rules(), "rg -r n 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_replace_bundled() {
        let result = eval_rules(rg_rules(), "rg -inr 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_replace_long() {
        let result = eval_rules(rg_rules(), "rg --replace='$1' 'v(\\d+)' file.txt");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_line_numbers() {
        let result = eval_rules(rg_rules(), "rg -n 'rel=' file.html");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
