//! Deny rule for `sd` chained with other commands.

use crate::prelude::*;

/// Deny `sd` when part of a compound command.
pub fn sd_rules() -> Vec<BashRule> {
    vec![sd__chained()]
}

/// Deny `sd` chained with other commands; point at built-in `Edit`.
fn sd__chained() -> BashRule {
    BashRule {
        condition: Some(|ctx| ctx.complete.is_chained()),
        ..BashRule::new(
            "sd__chained",
            "sd",
            Outcome::deny(
                "Chained `sd` is blocked. Use the built-in `Edit` tool instead (`replace_all: true` for repeated matches)",
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn sd_and_sd() {
        let result = eval_rules(sd_rules(), "sd -F 'a' 'b' x.rs && sd -F 'c' 'd' x.rs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sd_and_rg() {
        let result = eval_rules(sd_rules(), "sd -F 'a' 'b' x.rs && rg -n 'a' .");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn rg_or_sd() {
        let result = eval_rules(sd_rules(), "rg -q 'a' x.rs || sd 'a' 'b' x.rs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sd_semicolon_echo() {
        let result = eval_rules(sd_rules(), "sd 'a' 'b' x.rs ; echo done");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn standalone_sd() {
        let result = eval_rules(sd_rules(), "sd -F 'a' 'b' x.rs");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn piped_sd() {
        let result = eval_rules(sd_rules(), "cat x.rs | sd 'a' 'b'");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }
}
