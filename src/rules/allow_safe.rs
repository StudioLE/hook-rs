//! Allow rules for read-only and side-effect-free commands.

use crate::prelude::*;

const SAFE_COMMANDS: &[&str] = &[
    "base64", "cat", "column", "cut", "echo", "fmt", "grep", "head", "jq", "less", "ls", "rg",
    "tail", "tr", "tree", "uniq", "wc", "xxd",
];

/// Rules for safe read-only commands, with denials for commands that can write or execute.
#[must_use]
pub fn safe_rules() -> Vec<SimpleRule> {
    let mut rules: Vec<SimpleRule> = SAFE_COMMANDS
        .iter()
        .map(|cmd| SimpleRule::new(*cmd, *cmd, Outcome::allow(format!("Safe command: {cmd}"))))
        .collect();
    rules.push(awk());
    rules.push(sed());
    rules.push(sort__cmd());
    rules.push(tee());
    rules.push(yq());
    rules
}

/// Deny `awk` (can execute via `system()`).
fn awk() -> SimpleRule {
    SimpleRule::new(
        "awk",
        "awk",
        Outcome::deny("awk can execute commands via system()"),
    )
}

/// Allow `sed` without `-i`/`--in-place`.
fn sed() -> SimpleRule {
    SimpleRule {
        id: "sed".to_owned(),
        prefix: "sed".to_owned(),
        without_any: Some(vec!["-i".to_owned(), "--in-place".to_owned()]),
        outcome: Outcome::allow("Safe command: sed (no in-place edit)"),
        ..Default::default()
    }
}

/// Allow `sort` without `-o`/`--output`.
fn sort__cmd() -> SimpleRule {
    SimpleRule {
        id: "sort".to_owned(),
        prefix: "sort".to_owned(),
        without_any: Some(vec!["-o".to_owned(), "--output".to_owned()]),
        outcome: Outcome::allow("Safe command: sort (no output file)"),
        ..Default::default()
    }
}

/// Deny `tee` (writes to files).
fn tee() -> SimpleRule {
    SimpleRule::new("tee", "tee", Outcome::deny("tee writes to files"))
}

/// Allow `yq` without `-i`/`--in-place`.
fn yq() -> SimpleRule {
    SimpleRule {
        id: "yq".to_owned(),
        prefix: "yq".to_owned(),
        without_any: Some(vec!["-i".to_owned(), "--in-place".to_owned()]),
        outcome: Outcome::allow("Safe command: yq (no in-place edit)"),
        ..Default::default()
    }
}
