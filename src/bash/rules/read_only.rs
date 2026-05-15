//! Allow rules for read-only commands.

use crate::prelude::*;

const READ_ONLY_COMMANDS: &[&str] = &[
    "base64", "basename", "cat", "column", "command", "cut", "diff", "dirname", "echo", "file",
    "fmt", "grep", "head", "jq", "less", "ls", "readlink", "realpath", "rg", "stat", "tail", "tr",
    "tree", "type", "uniq", "wc", "which", "xxd",
];

/// Rules for read-only commands.
#[must_use]
pub fn read_only_rules() -> Vec<BashRule> {
    let mut rules: Vec<BashRule> = READ_ONLY_COMMANDS
        .iter()
        .map(|cmd| BashRule::new(*cmd, *cmd, Outcome::allow(format!("Read-only `{cmd}`"))))
        .collect();
    rules.push(sort__cmd());
    rules.push(yq());
    rules
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
    fn for_loop_rg_basename() {
        let cmd = r#"for f in src/bash/rules/snapshots/*git_deny*.snap; do echo "=== $(basename $f) ==="; rg "decision:" "$f"; done"#;
        let result = eval_rules(read_only_rules(), cmd);
        let outcome = expect_outcome(result);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
