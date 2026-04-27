//! Rules for `sops` invocations.
//!
//! - Deny `sops decrypt`, `sops -d`, `sops --decrypt`: print plaintext to stdout
//! - Deny `sops exec-env` / `sops exec-file` when the wrapped command reads
//!   secrets (dumps env, redirects, substitutes, etc.)
//! - Ask for `sops exec-env` / `sops exec-file` otherwise: legitimate use
//!   (e.g. `curl -H "Authorization: Bearer $TOKEN"`) requires user approval
//!
//! # Limitations
//!
//! Detection scans the wrapped command for known read-tool tokens and unsafe
//! shell metacharacters. The scan cannot see through:
//!
//! - Opaque binaries: `sops exec-env secrets.yaml mybinary` falls through to
//!   ASK even if `mybinary` internally dumps env
//! - Full-path invocations: `/usr/bin/env` is not matched against `env`
//! - Aliases or shell functions resolved at runtime
//!
//! Pipes (`|`) alone do not trigger DENY: `curl ... | jq .field` is permitted
//! to ASK because the pipe target may legitimately consume non-secret output.

use crate::prelude::*;

const DECRYPT_REASON: &str = "`sops` decryption is blocked. Exposes secrets";

const EXEC_DENY_REASON: &str = "`sops exec-env` / `sops exec-file` with a command that would expose secrets is blocked. \
                                Reading the env (`env`, `printenv`, `set`, `export`), printing the file (`cat`, `echo`, `printf`, `head`, `tail`, `od`, `xxd`, `strings`, `base64`), redirecting output (`>`, `>>`, `tee`), or substituting (`$(...)`, backticks) would surface plaintext to context";

const EXEC_ASK_REASON: &str = "`sops exec-env` / `sops exec-file` requires approval. The wrapped command receives plaintext secrets";

/// Tokens that, if used as the program name inside the wrapped command,
/// indicate an attempt to read decrypted secrets.
const READ_TOOLS: &[&str] = &[
    "env", "printenv", "set", "export", "cat", "echo", "printf", "head", "tail", "less", "more",
    "od", "xxd", "strings", "base64", "tee",
];

/// Build all `sops` rules.
pub fn sops_rules() -> Vec<BashRule> {
    vec![
        sops_exec_env__reads_secrets(),
        sops_exec_file__reads_secrets(),
        sops_exec_env__ask(),
        sops_exec_file__ask(),
        sops_decrypt(),
        sops_d(),
    ]
}

fn sops_exec_env__reads_secrets() -> BashRule {
    BashRule {
        condition: Some(exec_reads_secrets),
        ..BashRule::new(
            "sops_exec_env__reads_secrets",
            "sops exec-env",
            Outcome::deny(EXEC_DENY_REASON),
        )
    }
}

fn sops_exec_file__reads_secrets() -> BashRule {
    BashRule {
        condition: Some(exec_reads_secrets),
        ..BashRule::new(
            "sops_exec_file__reads_secrets",
            "sops exec-file",
            Outcome::deny(EXEC_DENY_REASON),
        )
    }
}

fn sops_exec_env__ask() -> BashRule {
    BashRule::new(
        "sops_exec_env__ask",
        "sops exec-env",
        Outcome::ask(EXEC_ASK_REASON),
    )
}

fn sops_exec_file__ask() -> BashRule {
    BashRule::new(
        "sops_exec_file__ask",
        "sops exec-file",
        Outcome::ask(EXEC_ASK_REASON),
    )
}

fn sops_decrypt() -> BashRule {
    BashRule::new(
        "sops_decrypt",
        "sops decrypt",
        Outcome::deny(DECRYPT_REASON),
    )
}

fn sops_d() -> BashRule {
    BashRule {
        id: "sops_d".to_owned(),
        command: "sops".to_owned(),
        with_any: Some(vec![Arg::new("-d"), Arg::new("--decrypt")]),
        outcome: Outcome::deny(DECRYPT_REASON),
        ..Default::default()
    }
}

/// Detect whether the command wrapped by `sops exec-env` / `sops exec-file`
/// would read secrets out of the decrypted env or file.
///
/// - The first arg is the subcommand (`exec-env` or `exec-file`); skip it
/// - Each remaining arg is fed back through [`BashParser`] so the inner
///   command is structurally analyzed rather than tokenized ad-hoc
fn exec_reads_secrets(simple: &SimpleContext, _: &CompleteContext, _: &Settings) -> bool {
    simple.args.iter().skip(1).any(|arg| arg_reads_secrets(arg))
}

/// Re-parse a single argument as a shell command and inspect every parsed
/// [`SimpleContext`] within it.
///
/// Returns true if any of the following hold:
///
/// - The parser refused the inner string with [`SkipReason::UnsafeRedirect`]
///   (e.g. `> /tmp/leak`), [`SkipReason::ProcessSubstitution`], or
///   [`SkipReason::CommandNameSubstitution`]: secrets would be redirected,
///   substituted, or used to pick a program at runtime
/// - Any inner command's name is in [`READ_TOOLS`]
/// - Any inner command contains a substitution (`$(...)`, backticks)
///
/// Unparseable args (e.g. plain file paths like `secrets.yaml`) are ignored:
/// the worst case is they fall through to ASK, which is the desired default.
fn arg_reads_secrets(arg: &str) -> bool {
    let unquoted = unquote_str(arg);
    match BashParser::new().parse(&unquoted) {
        Ok(context) => context
            .all_commands()
            .any(|c| READ_TOOLS.contains(&c.name.as_str()) || c.contains_substitution),
        Err(report) => matches!(
            report.current_context(),
            ParseError::Skip(
                SkipReason::UnsafeRedirect
                    | SkipReason::ProcessSubstitution
                    | SkipReason::CommandNameSubstitution
            )
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn sops_exec_env_dump_env() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml 'env | grep TOKEN'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_env_bare_env() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml env");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_env_printenv() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml printenv");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_env_echo_var() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml 'echo $TOKEN'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_env_redirect_to_file() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml 'mycmd > /tmp/leak.txt'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_env_substitution() {
        let outcome = evaluate_expect_outcome("sops exec-env secrets.yaml 'mycmd $(env)'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_env_curl_with_token() {
        let outcome = evaluate_expect_outcome(
            "sops exec-env secrets.yaml 'curl -H \"Authorization: Bearer $TOKEN\" https://example.com'",
        );
        assert_eq!(outcome.decision, Decision::Ask);
    }

    #[test]
    fn sops_exec_file_cat_placeholder() {
        let outcome = evaluate_expect_outcome("sops exec-file secrets.yaml 'cat {}'");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_exec_file_apply_placeholder() {
        let outcome = evaluate_expect_outcome("sops exec-file secrets.yaml 'kubectl apply -f {}'");
        assert_eq!(outcome.decision, Decision::Ask);
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
    fn sops_decrypt_long_flag() {
        let outcome = evaluate_expect_outcome("sops --decrypt secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_decrypt_chained() {
        let outcome = evaluate_expect_outcome("sops decrypt secrets.yaml | grep token");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_decrypt_in_chain() {
        let outcome = evaluate_expect_outcome("git pull && sops decrypt secrets.yaml");
        assert_eq!(outcome.decision, Decision::Deny);
    }

    #[test]
    fn sops_d_output_file() {
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
