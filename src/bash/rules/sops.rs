//! Deny rules for `sops` invocations that expose decrypted secrets.

use crate::prelude::*;

const REASON: &str = "`sops` decryption is blocked. Plaintext secrets would be exposed to context";

/// Deny `sops` subcommands that surface plaintext secrets.
pub fn sops_rules() -> Vec<BashRule> {
    vec![sops_exec_env(), sops_exec_file(), sops_decrypt(), sops_d()]
}

/// Deny `sops exec-env`.
fn sops_exec_env() -> BashRule {
    BashRule::new("sops_exec_env", "sops exec-env", Outcome::deny(REASON))
}

/// Deny `sops exec-file`.
fn sops_exec_file() -> BashRule {
    BashRule::new("sops_exec_file", "sops exec-file", Outcome::deny(REASON))
}

/// Deny `sops decrypt`.
fn sops_decrypt() -> BashRule {
    BashRule::new("sops_decrypt", "sops decrypt", Outcome::deny(REASON))
}

/// Deny `sops -d` and `sops --decrypt`.
fn sops_d() -> BashRule {
    BashRule {
        id: "sops_d".to_owned(),
        command: "sops".to_owned(),
        with_any: Some(vec![Arg::new("-d"), Arg::new("--decrypt")]),
        outcome: Outcome::deny(REASON),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn sops_exec_env() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml 'env | grep TOKEN'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_file() {
        let outcome = evaluate_expect_outcome("sops exec-file secrets.yaml 'cat {}'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_decrypt() {
        let outcome = evaluate_expect_outcome("sops decrypt secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_d() {
        let outcome = evaluate_expect_outcome("sops -d secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_decrypt__long_flag() {
        let outcome = evaluate_expect_outcome("sops --decrypt secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_decrypt__chained() {
        let outcome = evaluate_expect_outcome("sops decrypt secrets.yaml | grep token");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_decrypt__in_chain() {
        let outcome = evaluate_expect_outcome("git pull && sops decrypt secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_d__output_file() {
        let outcome = evaluate_expect_outcome("sops --decrypt --output /tmp/x.yaml secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_encrypt() {
        let reason = evaluate_expect_skip("sops encrypt secrets.yaml");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn sops_edit() {
        let reason = evaluate_expect_skip("sops edit secrets.yaml");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn sops_updatekeys() {
        let reason = evaluate_expect_skip("sops updatekeys secrets.yaml");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn sops_rotate() {
        let reason = evaluate_expect_skip("sops rotate -i secrets.yaml");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn sops_set() {
        let reason = evaluate_expect_skip("sops set secrets.yaml '[\"key\"]' '\"value\"'");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn sops_publish() {
        let reason = evaluate_expect_skip("sops publish secrets.yaml");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn sops_bare() {
        let reason = evaluate_expect_skip("sops");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn echo_sops_decrypt() {
        let outcome = evaluate_expect_outcome("echo sops decrypt is blocked");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn rg_sops_decrypt() {
        let outcome = evaluate_expect_outcome("rg 'sops decrypt' README.md");
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
