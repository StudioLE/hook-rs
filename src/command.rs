//! Shell command parser and AST types.
//!
//! - Tokenizes and parses shell commands via [`brush_parser`]
//! - Produces a [`ParsedCommand`] AST for check evaluation

use brush_parser::ast::{
    self, AndOr, Command, CommandPrefixOrSuffixItem, CompoundCommand, CompoundList, IoRedirect,
};
use brush_parser::{ParserOptions, SourceInfo};

/// Parsed representation of a shell command string.
#[derive(Debug)]
pub struct ParsedCommand {
    /// Original command string.
    pub raw: String,
    /// Top-level semicolon-separated statement lists.
    pub and_or_lists: Vec<AndOrContext>,
}

/// A single semicolon-separated statement containing pipelines joined by `&&` or `||`.
#[derive(Debug)]
pub struct AndOrContext {
    /// Pipeline items in this statement.
    pub items: Vec<PipelineItem>,
}

/// A pipeline segment with an optional logical connector to the previous segment.
#[derive(Debug)]
pub struct PipelineItem {
    /// Logical connector (`&&` or `||`) linking to the previous item.
    pub connector: Option<Connector>,
    /// Commands piped together in this segment.
    pub commands: Vec<CommandContext>,
}

/// A single simple command extracted from the shell AST.
#[derive(Debug)]
pub struct CommandContext {
    /// Command name (first word).
    pub name: String,
    /// Positional arguments.
    pub args: Vec<String>,
    /// Whether the command has a heredoc redirect.
    pub has_heredoc: bool,
}

/// Logical connector between pipeline items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connector {
    And,
    Or,
}

impl ParsedCommand {
    /// Iterate over all [`CommandContext`] in the parsed command.
    pub fn all_commands(&self) -> impl Iterator<Item = &CommandContext> {
        self.and_or_lists
            .iter()
            .flat_map(|aol| &aol.items)
            .flat_map(|pi| &pi.commands)
    }

    /// True if the command is a single simple command with no chaining or piping.
    #[must_use]
    pub fn is_standalone(&self) -> bool {
        self.and_or_lists.first().is_some_and(|aol| {
            aol.items.len() == 1 && aol.items.first().is_some_and(|pi| pi.commands.len() == 1)
        }) && self.and_or_lists.len() == 1
    }
}

/// Parse a shell command string into a [`ParsedCommand`].
#[must_use]
pub fn parse(command: &str) -> Option<ParsedCommand> {
    let tokens = brush_parser::tokenize_str(command).ok()?;
    let program =
        brush_parser::parse_tokens(&tokens, &ParserOptions::default(), &SourceInfo::default())
            .ok()?;
    let mut and_or_lists = Vec::new();
    for cc in &program.complete_commands {
        walk_compound_list(cc, &mut and_or_lists);
    }
    Some(ParsedCommand {
        raw: command.to_owned(),
        and_or_lists,
    })
}

/// Remove surrounding quotes from a string.
#[must_use]
pub fn unquote(s: &str) -> String {
    brush_parser::unquote_str(s)
}

fn walk_compound_list(list: &CompoundList, out: &mut Vec<AndOrContext>) {
    for item in &list.0 {
        walk_and_or_list(&item.0, out);
    }
}

fn walk_and_or_list(aol: &ast::AndOrList, out: &mut Vec<AndOrContext>) {
    let mut items = Vec::new();
    let first_commands = walk_pipeline(&aol.first, out);
    if !first_commands.is_empty() {
        items.push(PipelineItem {
            connector: None,
            commands: first_commands,
        });
    }
    for ao in &aol.additional {
        let (connector, pipeline) = match ao {
            AndOr::And(p) => (Connector::And, p),
            AndOr::Or(p) => (Connector::Or, p),
        };
        let commands = walk_pipeline(pipeline, out);
        if !commands.is_empty() {
            items.push(PipelineItem {
                connector: Some(connector),
                commands,
            });
        }
    }
    if !items.is_empty() {
        out.push(AndOrContext { items });
    }
}

fn walk_pipeline(p: &ast::Pipeline, out: &mut Vec<AndOrContext>) -> Vec<CommandContext> {
    let mut commands = Vec::new();
    for cmd in &p.seq {
        match cmd {
            Command::Simple(sc) => {
                if let Some(ctx) = extract_simple_command(sc) {
                    commands.push(ctx);
                }
            }
            Command::Compound(cc, _) => {
                walk_compound_command(cc, out);
            }
            Command::Function(_) | Command::ExtendedTest(_) => {}
        }
    }
    commands
}

fn walk_compound_command(cc: &CompoundCommand, out: &mut Vec<AndOrContext>) {
    match cc {
        CompoundCommand::BraceGroup(bg) => walk_compound_list(&bg.list, out),
        CompoundCommand::Subshell(sub) => walk_compound_list(&sub.list, out),
        CompoundCommand::ForClause(f) => walk_compound_list(&f.body.list, out),
        CompoundCommand::ArithmeticForClause(f) => walk_compound_list(&f.body.list, out),
        CompoundCommand::WhileClause(w) | CompoundCommand::UntilClause(w) => {
            walk_compound_list(&w.0, out);
            walk_compound_list(&w.1.list, out);
        }
        CompoundCommand::IfClause(ic) => {
            walk_compound_list(&ic.condition, out);
            walk_compound_list(&ic.then, out);
            if let Some(elses) = &ic.elses {
                for else_clause in elses {
                    if let Some(cond) = &else_clause.condition {
                        walk_compound_list(cond, out);
                    }
                    walk_compound_list(&else_clause.body, out);
                }
            }
        }
        CompoundCommand::CaseClause(cc) => {
            for case_item in &cc.cases {
                if let Some(cmd_list) = &case_item.cmd {
                    walk_compound_list(cmd_list, out);
                }
            }
        }
        CompoundCommand::Arithmetic(_) => {}
    }
}

fn extract_simple_command(sc: &ast::SimpleCommand) -> Option<CommandContext> {
    let name_word = sc.word_or_name.as_ref()?;
    let name = unquote(&name_word.value);
    let mut args = Vec::new();
    let mut has_heredoc = false;
    if let Some(suffix) = &sc.suffix {
        for item in &suffix.0 {
            match item {
                CommandPrefixOrSuffixItem::Word(w) => args.push(w.value.clone()),
                CommandPrefixOrSuffixItem::IoRedirect(IoRedirect::HereDocument(..)) => {
                    has_heredoc = true;
                }
                _ => {}
            }
        }
    }
    if let Some(prefix) = &sc.prefix {
        for item in &prefix.0 {
            if matches!(
                item,
                CommandPrefixOrSuffixItem::IoRedirect(IoRedirect::HereDocument(..))
            ) {
                has_heredoc = true;
            }
        }
    }
    Some(CommandContext {
        name,
        args,
        has_heredoc,
    })
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "test assertions — panic is a test failure"
)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let p = parse("git status").expect("should parse");
        assert_eq!(p.and_or_lists.len(), 1);
        assert_eq!(p.and_or_lists[0].items.len(), 1);
        let cmd = &p.and_or_lists[0].items[0].commands[0];
        assert_eq!(cmd.name, "git");
        assert_eq!(cmd.args[0], "status");
    }

    #[test]
    fn parse_and_chain() {
        let p = parse("ls && git status").expect("should parse");
        assert_eq!(p.and_or_lists.len(), 1);
        assert_eq!(p.and_or_lists[0].items.len(), 2);
        assert_eq!(p.and_or_lists[0].items[0].connector, None);
        assert_eq!(p.and_or_lists[0].items[1].connector, Some(Connector::And));
        assert_eq!(p.and_or_lists[0].items[1].commands[0].name, "git");
    }

    #[test]
    fn parse_or_chain() {
        let p = parse("false || git stash clear").expect("should parse");
        assert_eq!(p.and_or_lists[0].items[1].connector, Some(Connector::Or));
        assert_eq!(p.and_or_lists[0].items[1].commands[0].name, "git");
    }

    #[test]
    fn parse_pipe() {
        let p = parse("git log | head -5").expect("should parse");
        assert_eq!(p.and_or_lists.len(), 1);
        assert_eq!(p.and_or_lists[0].items.len(), 1);
        assert_eq!(p.and_or_lists[0].items[0].commands.len(), 2);
        assert_eq!(p.and_or_lists[0].items[0].commands[0].name, "git");
        assert_eq!(p.and_or_lists[0].items[0].commands[1].name, "head");
    }

    #[test]
    fn parse_semicolon() {
        let p = parse("git status ; git log").expect("should parse");
        assert_eq!(p.and_or_lists.len(), 2);
    }

    #[test]
    fn parse_heredoc() {
        let p = parse("cargo insta review <<EOF\na\nEOF").expect("should parse");
        assert!(p.and_or_lists[0].items[0].commands[0].has_heredoc);
    }

    #[test]
    fn parse_no_heredoc() {
        let p = parse("cargo insta review").expect("should parse");
        assert!(!p.and_or_lists[0].items[0].commands[0].has_heredoc);
    }

    #[test]
    fn parse_for_loop() {
        let p = parse("for f in *.tmp; do echo $f; done").expect("should parse");
        assert!(p.all_commands().any(|cmd| cmd.name == "echo"));
    }

    #[test]
    fn parse_if_then() {
        let p = parse("if true; then echo hello; fi").expect("should parse");
        assert!(p.all_commands().any(|cmd| cmd.name == "echo"));
    }

    #[test]
    fn unquote_single() {
        assert_eq!(unquote("'hello'"), "hello");
    }

    #[test]
    fn unquote_double() {
        assert_eq!(unquote("\"hello\""), "hello");
    }

    #[test]
    fn unquote_bare() {
        assert_eq!(unquote("hello"), "hello");
    }

    #[test]
    fn no_space_before_and() {
        let p = parse("git commit -m 'msg'&& git push").expect("should parse");
        assert_eq!(p.and_or_lists.len(), 1);
        assert_eq!(p.and_or_lists[0].items.len(), 2);
        assert_eq!(p.and_or_lists[0].items[1].commands[0].name, "git");
    }
}
