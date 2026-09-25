//! Allow rules for read-only commands.

use crate::prelude::*;

const READ_ONLY_COMMANDS: &[&str] = &[
    "base64", "basename", "cat", "column", "command", "cut", "diff", "dirname", "echo", "eza",
    "file", "fmt", "grep", "head", "jq", "less", "ls", "pwd", "readlink", "realpath", "stat",
    "tail", "tiktoken", "tr", "tree", "type", "uniq", "wc", "which", "xxd",
];

/// Rules for read-only commands.
#[must_use]
pub fn read_only_rules() -> Vec<BashRule> {
    let mut rules: Vec<BashRule> = READ_ONLY_COMMANDS
        .iter()
        .map(|cmd| BashRule::new(*cmd, *cmd, Outcome::allow(format!("Read-only `{cmd}`"))))
        .collect();
    rules.push(bat());
    rules.push(sort__cmd());
    rules.push(yq());
    rules
}

/// Allow `bat` without pager, preprocessor, cache, or config-writing options.
///
/// - `--pager` and `--paging always|auto` can spawn an arbitrary program
/// - `--lessopen` runs the `LESSOPEN` preprocessor
/// - `cache` and `--generate-config-file` write files
/// - Env var prefixes (`BAT_PAGER`, `LESSOPEN`, `BAT_CONFIG_PATH`) can do the same
fn bat() -> BashRule {
    BashRule {
        id: "bat".to_owned(),
        command: "bat".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("--pager"),
            ArgMatcher::new("--paging").value("{always,auto}"),
            ArgMatcher::new("--lessopen"),
            ArgMatcher::new("--generate-config-file"),
            ArgMatcher::new("cache"),
        ]),
        condition: Some(|ctx| ctx.simple.env_vars.is_empty()),
        outcome: Outcome::allow("Read-only `bat`"),
        ..Default::default()
    }
}

/// Allow `sort` without `-o`/`--output`.
fn sort__cmd() -> BashRule {
    BashRule {
        id: "sort".to_owned(),
        command: "sort".to_owned(),
        without_any: Some(vec![ArgMatcher::new("-o"), ArgMatcher::new("--output")]),
        outcome: Outcome::allow("Read-only `sort`"),
        ..Default::default()
    }
}

/// Allow `yq` without `-i`/`--in-place`.
fn yq() -> BashRule {
    BashRule {
        id: "yq".to_owned(),
        command: "yq".to_owned(),
        without_any: Some(vec![ArgMatcher::new("-i"), ArgMatcher::new("--in-place")]),
        outcome: Outcome::allow("Read-only `yq`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn bat_file() {
        let result = eval_rules(read_only_rules(), "bat -p -r 10:20 src/main.rs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn bat_paging_never() {
        let result = eval_rules(read_only_rules(), "bat --paging=never src/main.rs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn bat_no_lessopen() {
        let result = eval_rules(read_only_rules(), "bat --no-lessopen src/main.rs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn bat_pager() {
        let result = eval_rules(
            read_only_rules(),
            "bat --paging=always --pager 'sh -c id' src/main.rs",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn bat_paging_always() {
        let result = eval_rules(read_only_rules(), "bat --paging always src/main.rs");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn bat_lessopen() {
        let result = eval_rules(read_only_rules(), "bat --lessopen src/main.rs");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn bat_cache_build() {
        let result = eval_rules(read_only_rules(), "bat cache --build");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn bat_generate_config_file() {
        let result = eval_rules(read_only_rules(), "bat --generate-config-file");
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn bat_env_prefix() {
        let result = eval_rules(
            read_only_rules(),
            "BAT_PAGER='sh -c id' bat --paging=never src/main.rs",
        );
        let reason = expect_skip(result);
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn diff_two_files() {
        let result = eval_rules(read_only_rules(), "diff a.snap a.snap.new");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn diff_piped_head() {
        let result = eval_rules(read_only_rules(), "diff a.snap a.snap.new | head -50");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn eza_long_tree() {
        let result = eval_rules(read_only_rules(), "eza -la --tree --level 2 --git src docs");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn pwd_chained() {
        let result = eval_rules(read_only_rules(), "pwd && ls -la");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn tiktoken_files() {
        let result = eval_rules(read_only_rules(), "tiktoken README.md docs/*.md");
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn for_loop_grep_basename() {
        let cmd = r#"for f in src/bash/rules/snapshots/*git_deny*.snap; do echo "=== $(basename $f) ==="; grep "decision:" "$f"; done"#;
        let result = eval_rules(read_only_rules(), cmd);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
