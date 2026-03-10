//! Allow rules for read-only and side-effect-free commands.

use crate::prelude::*;

const SAFE_COMMANDS: &[&str] = &[
    "cat", "column", "cut", "echo", "fmt", "grep", "head", "jq", "less", "ls", "rg", "tail", "tr",
    "tree", "uniq", "wc", "xxd",
];

/// Rules for safe read-only commands, with denials for commands that can write or execute.
#[must_use]
pub fn safe_rules() -> Vec<SimpleRule> {
    let mut rules: Vec<SimpleRule> = SAFE_COMMANDS
        .iter()
        .map(|cmd| SimpleRule::new(*cmd, Outcome::allow(format!("Safe command: {cmd}"))))
        .collect();
    rules.push(SimpleRule::new(
        "awk",
        Outcome::deny("awk can execute commands via system()"),
    ));
    rules.push(SimpleRule {
        prefix: "sed".to_owned(),
        without_option: Some(vec!["-i".to_owned(), "--in-place".to_owned()]),
        outcome: Outcome::allow("Safe command: sed (no in-place edit)"),
        ..Default::default()
    });
    rules.push(SimpleRule {
        prefix: "sort".to_owned(),
        without_option: Some(vec!["-o".to_owned(), "--output".to_owned()]),
        outcome: Outcome::allow("Safe command: sort (no output file)"),
        ..Default::default()
    });
    rules.push(SimpleRule::new("tee", Outcome::deny("tee writes to files")));
    rules.push(SimpleRule {
        prefix: "yq".to_owned(),
        without_option: Some(vec!["-i".to_owned(), "--in-place".to_owned()]),
        outcome: Outcome::allow("Safe command: yq (no in-place edit)"),
        ..Default::default()
    });
    rules
}
