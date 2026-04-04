//! Rules for `journalctl` commands, allowing read-only operations.

use crate::prelude::*;

/// Allow read-only `journalctl`, let mutating operations pass through.
pub fn journalctl_rules() -> Vec<BashRule> {
    vec![journalctl__read_only()]
}

/// Allow read-only `journalctl` (no mutating flags).
fn journalctl__read_only() -> BashRule {
    BashRule {
        id: "journalctl__read_only".to_owned(),
        command: "journalctl".to_owned(),
        without_any: Some(vec![
            Arg::new("--vacuum-size"),
            Arg::new("--vacuum-files"),
            Arg::new("--vacuum-time"),
            Arg::new("--rotate"),
            Arg::new("--flush"),
            Arg::new("--sync"),
            Arg::new("--relinquish-var"),
            Arg::new("--smart-relinquish-var"),
            Arg::new("--setup-keys"),
        ]),
        outcome: Outcome::allow("Read-only journalctl"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    /// Read-only log viewing.
    #[test]
    fn journalctl__read() {
        let outcome = evaluate_expect_outcome("journalctl --since today -u sshd.service");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Read-only with pipe to rg.
    #[test]
    fn journalctl__piped_to_rg() {
        let outcome = evaluate_expect_outcome(
            "journalctl --since \"today\" -u uupd.service --no-pager 2>&1 | rg '\"(ERROR|WARN)'",
        );
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Mutating flags pass through to default permission handling.
    #[test]
    fn journalctl__vacuum_size() {
        let reason = evaluate_expect_skip("journalctl --vacuum-size=500M");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Rotate passes through to default permission handling.
    #[test]
    fn journalctl__rotate() {
        let reason = evaluate_expect_skip("journalctl --rotate");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Plain journalctl with no flags.
    #[test]
    fn journalctl__bare() {
        let outcome = evaluate_expect_outcome("journalctl");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
