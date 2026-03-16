//! Allow rules for read-only and side-effect-free commands.

use crate::prelude::*;

const SAFE_COMMANDS: &[&str] = &[
    "base64", "basename", "cat", "column", "command", "cut", "dirname", "echo", "file", "fmt",
    "grep", "head", "jq", "less", "ls", "readlink", "realpath", "rg", "stat", "tail", "tr", "tree",
    "type", "uniq", "wc", "which", "xxd",
];

/// Rules for safe read-only commands, with denials for commands that can write or execute.
#[must_use]
pub fn safe_rules() -> Vec<BashRule> {
    let mut rules: Vec<BashRule> = SAFE_COMMANDS
        .iter()
        .map(|cmd| BashRule::new(*cmd, *cmd, Outcome::allow(format!("Safe command: {cmd}"))))
        .collect();
    rules.push(awk());
    rules.push(sed());
    rules.push(sort__cmd());
    rules.push(tee());
    rules.push(yq());
    rules
}

/// Deny `awk` (can execute via `system()`).
fn awk() -> BashRule {
    BashRule::new(
        "awk",
        "awk",
        Outcome::deny("awk can execute commands via system()"),
    )
}

/// Allow `sed` without `-i`/`--in-place`.
fn sed() -> BashRule {
    BashRule {
        id: "sed".to_owned(),
        command: "sed".to_owned(),
        without_any: Some(vec![Arg::new("-i"), Arg::new("--in-place")]),
        outcome: Outcome::allow("Safe command: sed (no in-place edit)"),
        ..Default::default()
    }
}

/// Allow `sort` without `-o`/`--output`.
fn sort__cmd() -> BashRule {
    BashRule {
        id: "sort".to_owned(),
        command: "sort".to_owned(),
        without_any: Some(vec![Arg::new("-o"), Arg::new("--output")]),
        outcome: Outcome::allow("Safe command: sort (no output file)"),
        ..Default::default()
    }
}

/// Deny `tee` (writes to files).
fn tee() -> BashRule {
    BashRule::new("tee", "tee", Outcome::deny("tee writes to files"))
}

/// Allow `yq` without `-i`/`--in-place`.
fn yq() -> BashRule {
    BashRule {
        id: "yq".to_owned(),
        command: "yq".to_owned(),
        without_any: Some(vec![Arg::new("-i"), Arg::new("--in-place")]),
        outcome: Outcome::allow("Safe command: yq (no in-place edit)"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn for_loop_grep_basename() {
        let cmd = r#"for f in src/bash/rules/snapshots/*git_deny*.snap; do echo "=== $(basename $f) ==="; grep "decision:" "$f"; done"#;
        let outcome = evaluate_expect_outcome(cmd);
        assert_eq!(outcome.decision, Decision::Allow);
    }
}
