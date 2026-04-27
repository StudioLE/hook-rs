//! Allow rules for read-only commands.

use crate::prelude::*;

const READ_ONLY_COMMANDS: &[&str] = &[
    "base64", "basename", "cat", "column", "command", "cut", "dirname", "echo", "file", "fmt",
    "grep", "head", "jq", "less", "ls", "readlink", "realpath", "rg", "stat", "tail", "tr", "tree",
    "type", "uniq", "wc", "which", "xxd",
];

/// Rules for read-only commands.
#[must_use]
pub fn read_only_rules() -> Vec<BashRule> {
    let mut rules: Vec<BashRule> = READ_ONLY_COMMANDS
        .iter()
        .map(|cmd| BashRule::new(*cmd, *cmd, Outcome::allow(format!("Read-only `{cmd}`"))))
        .collect();
    rules.push(sed());
    rules.push(sort__cmd());
    rules.push(yq());
    rules
}

/// Allow `sed` without `-i`/`--in-place`.
fn sed() -> BashRule {
    BashRule {
        id: "sed".to_owned(),
        command: "sed".to_owned(),
        without_any: Some(vec![Arg::new("-i"), Arg::new("--in-place")]),
        outcome: Outcome::allow("Read-only `sed`"),
        ..Default::default()
    }
}

/// Allow `sort` without `-o`/`--output`.
fn sort__cmd() -> BashRule {
    BashRule {
        id: "sort".to_owned(),
        command: "sort".to_owned(),
        without_any: Some(vec![Arg::new("-o"), Arg::new("--output")]),
        outcome: Outcome::allow("Read-only `sort`"),
        ..Default::default()
    }
}

/// Allow `yq` without `-i`/`--in-place`.
fn yq() -> BashRule {
    BashRule {
        id: "yq".to_owned(),
        command: "yq".to_owned(),
        without_any: Some(vec![Arg::new("-i"), Arg::new("--in-place")]),
        outcome: Outcome::allow("Read-only `yq`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn for_loop_rg_basename() {
        let cmd = r#"for f in src/bash/rules/snapshots/*git_deny*.snap; do echo "=== $(basename $f) ==="; rg "decision:" "$f"; done"#;
        let outcome = evaluate_expect_outcome(cmd);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
