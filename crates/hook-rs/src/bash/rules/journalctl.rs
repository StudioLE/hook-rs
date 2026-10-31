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
            ArgMatcher::new("--vacuum-size"),
            ArgMatcher::new("--vacuum-files"),
            ArgMatcher::new("--vacuum-time"),
            ArgMatcher::new("--rotate"),
            ArgMatcher::new("--flush"),
            ArgMatcher::new("--sync"),
            ArgMatcher::new("--relinquish-var"),
            ArgMatcher::new("--smart-relinquish-var"),
            ArgMatcher::new("--setup-keys"),
        ]),
        outcome: Outcome::allow("Read-only `journalctl`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    /// Read-only log viewing.
    #[test]
    fn journalctl_read() {
        let result = eval_rules(
            journalctl_rules(),
            "journalctl --since today -u sshd.service",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    /// Read-only with pipe to rg.
    #[test]
    fn journalctl_piped_to_rg() {
        let result = eval_rules(
            journalctl_rules(),
            "journalctl --since \"today\" -u uupd.service --no-pager 2>&1 | rg '\"(ERROR|WARN)'",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    /// Mutating flags pass through to default permission handling.
    #[test]
    fn journalctl_vacuum_size() {
        let result = eval_rules(journalctl_rules(), "journalctl --vacuum-size=500M");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Rotate passes through to default permission handling.
    #[test]
    fn journalctl_rotate() {
        let result = eval_rules(journalctl_rules(), "journalctl --rotate");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    /// Plain journalctl with no flags.
    #[test]
    fn journalctl_bare() {
        let result = eval_rules(journalctl_rules(), "journalctl");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
