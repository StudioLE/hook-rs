//! Rules for cargo subcommands.
//!
//! Allows read-only subcommands and `cargo insta` subcommands that confine
//! writes to the project's default `target/` directory. The `--target-dir`
//! and `--out-dir` flags exclude a command from the allow list.
//!
//! Also denies `cargo insta review` with heredoc input, since heredocs fake
//! interactive accept/reject keystrokes.

use crate::prelude::*;

/// Subcommands that only read sources and write under `target/`.
const SAFE_SUBCOMMANDS: &[&str] = &["build", "check", "clippy", "doc", "test"];

/// `cargo insta` subcommands that read snapshots or run tests under `target/`.
const SAFE_INSTA_SUBCOMMANDS: &[&str] = &["accept", "pending-snapshots", "review", "test"];

/// Rules for `cargo` subcommands.
#[must_use]
pub fn cargo_rules() -> Vec<BashRule> {
    let mut rules: Vec<BashRule> = SAFE_SUBCOMMANDS
        .iter()
        .map(|sub| cargo_subcommand(sub))
        .collect();
    rules.extend(
        SAFE_INSTA_SUBCOMMANDS
            .iter()
            .map(|sub| cargo_insta_subcommand(sub)),
    );
    rules.push(cargo_insta_review__heredoc());
    rules
}

fn cargo_subcommand(sub: &str) -> BashRule {
    BashRule {
        id: format!("cargo_{sub}"),
        command: format!("cargo {sub}"),
        without_any: Some(vec![
            ArgMatcher::new("--target-dir"),
            ArgMatcher::new("--out-dir"),
        ]),
        outcome: Outcome::allow(format!("Safe `cargo {sub}`")),
        ..Default::default()
    }
}

fn cargo_insta_subcommand(sub: &str) -> BashRule {
    BashRule {
        id: format!("cargo_insta_{sub}"),
        command: format!("cargo insta {sub}"),
        without_any: Some(vec![
            ArgMatcher::new("--target-dir"),
            ArgMatcher::new("--out-dir"),
        ]),
        outcome: Outcome::allow(format!("Safe `cargo insta {sub}`")),
        ..Default::default()
    }
}

/// Deny `cargo insta review` with heredoc input.
fn cargo_insta_review__heredoc() -> BashRule {
    BashRule {
        id: "cargo_insta_review__heredoc".to_owned(),
        command: "cargo insta review".to_owned(),
        condition: Some(|ctx| ctx.simple.has_heredoc),
        outcome: Outcome::deny(
            "`cargo insta review` with heredoc input is blocked. \
             Alternatives: `cargo insta accept`, `cargo insta pending-snapshots`",
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn cargo_doc() {
        let result = eval_rules(cargo_rules(), "cargo doc");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_doc_with_flags() {
        let result = eval_rules(
            cargo_rules(),
            "cargo doc --no-deps --all-features --workspace",
        );
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_doc_with_env_prefix() {
        let cmd = r#"RUSTDOCFLAGS="-W missing-docs" cargo doc --no-deps --all-features --workspace 2>&1 | rg -B 1 "missing documentation" | head -200"#;
        let result = eval_rules(cargo_rules(), cmd);
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::OnlyAllowAll);
    }

    #[test]
    fn cargo_build() {
        let result = eval_rules(cargo_rules(), "cargo build");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_check() {
        let result = eval_rules(cargo_rules(), "cargo check --all-targets");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_clippy() {
        let result = eval_rules(cargo_rules(), "cargo clippy --all-targets --all-features");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_test() {
        let result = eval_rules(cargo_rules(), "cargo test");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_doc_target_dir() {
        let result = eval_rules(cargo_rules(), "cargo doc --target-dir /tmp/docs");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_doc_target_dir_equals() {
        let result = eval_rules(cargo_rules(), "cargo doc --target-dir=/tmp/docs");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_build_out_dir() {
        let result = eval_rules(cargo_rules(), "cargo build --out-dir ./bin");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_install() {
        let result = eval_rules(cargo_rules(), "cargo install --path .");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_publish() {
        let result = eval_rules(cargo_rules(), "cargo publish");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_run_release() {
        let result = eval_rules(cargo_rules(), "cargo run --release");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_insta_test() {
        let result = eval_rules(cargo_rules(), "cargo insta test");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_insta_review() {
        let result = eval_rules(cargo_rules(), "cargo insta review");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_insta_accept() {
        let result = eval_rules(cargo_rules(), "cargo insta accept");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_insta_pending_snapshots() {
        let result = eval_rules(cargo_rules(), "cargo insta pending-snapshots");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn cargo_insta_test_target_dir() {
        let result = eval_rules(cargo_rules(), "cargo insta test --target-dir /tmp/x");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn cargo_insta_review_heredoc_single_quoted() {
        let result = eval_rules(cargo_rules(), "cargo insta review 2>&1 <<'EOF'\na\nEOF");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cargo_insta_review_heredoc_unquoted() {
        let result = eval_rules(cargo_rules(), "cargo insta review <<EOF\na\nEOF");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cargo_insta_review_heredoc_double_quoted() {
        let result = eval_rules(cargo_rules(), "cargo insta review <<\"EOF\"\na\nEOF");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn cargo_insta_review_heredoc_dash() {
        let result = eval_rules(cargo_rules(), "cargo insta review <<-EOF\na\nEOF");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Deny);
    }
}
