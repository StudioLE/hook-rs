//! Shell command parser and AST types.
//!
//! - Tokenizes and parses shell commands via [`brush_parser`]
//! - Produces a [`CompleteContext`] AST for check evaluation

use brush_parser::ast::*;
use brush_parser::*;

/// Complete command context.
///
/// Example: `cd path/to/repo && git diff --stat HEAD~3 | head -5`
#[derive(Debug)]
pub struct CompleteContext {
    /// Original command string.
    pub raw: String,
    /// Command pipelines split by `&&` or `||`.
    pub children: Vec<PipelineContext>,
}

/// Multiple [`SimpleCommand`] in a `|` pipeline.
///
/// Example: `git diff --stat HEAD~3 | head -5`
#[derive(Debug)]
pub struct PipelineContext {
    /// Logical connector (`&&` or `||`) linking to the previous item.
    ///
    /// `None` for the first.
    pub connector: Option<Connector>,
    /// Individual commands piped together with `|`.
    pub children: Vec<SimpleContext>,
}

/// A simple command
///
/// Example: `git diff --stat HEAD~3`
#[derive(Debug)]
pub struct SimpleContext {
    /// Utility name
    pub name: String,
    /// Arguments
    pub args: Vec<String>,
    /// Whether the command has a heredoc redirect.
    pub has_heredoc: bool,
}

/// Logical connector between pipeline items.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Connector {
    And,
    Or,
}

impl CompleteContext {
    /// Parse a shell command string into a [`CompleteContext`].
    #[must_use]
    pub fn parse(command: &str) -> Option<CompleteContext> {
        let tokens = tokenize_str(command).ok()?;
        let program =
            parse_tokens(&tokens, &ParserOptions::default(), &SourceInfo::default()).ok()?;
        let aol = &program.complete_commands.first()?.0.first()?.0;
        let children: Vec<_> = std::iter::once((None, &aol.first))
            .chain(aol.additional.iter().map(split_and_or))
            .filter_map(pipeline_to_connector)
            .collect();
        Some(CompleteContext {
            raw: command.to_owned(),
            children,
        })
    }

    /// Iterate over all [`SimpleContext`] in the parsed command.
    pub fn all_commands(&self) -> impl Iterator<Item = &SimpleContext> {
        self.children.iter().flat_map(|pi| &pi.children)
    }

    /// True if the command is a single simple command with no chaining or piping.
    #[must_use]
    pub fn is_standalone(&self) -> bool {
        self.children.len() == 1
            && self
                .children
                .first()
                .is_some_and(|pi| pi.children.len() == 1)
    }
}

impl SimpleContext {
    /// Create a [`SimpleContext`] from a shell [`Command`].
    ///
    /// - Return `None` for non-simple commands (compound, function, etc.)
    fn from_command(cmd: &Command) -> Option<Self> {
        match cmd {
            Command::Simple(sc) => Self::from_simple_command(sc),
            _ => None,
        }
    }

    /// Create a [`SimpleContext`] from a [`SimpleCommand`](SimpleCommand).
    ///
    /// - Extract the command name and positional arguments
    /// - Detect heredoc redirects in prefix and suffix
    fn from_simple_command(sc: &SimpleCommand) -> Option<Self> {
        let name = unquote(&sc.word_or_name.as_ref()?.value);
        let args = sc
            .suffix
            .iter()
            .flat_map(|s| &s.0)
            .filter_map(|item| match item {
                CommandPrefixOrSuffixItem::Word(w) => Some(w.value.clone()),
                _ => None,
            })
            .collect();
        let has_heredoc = sc
            .suffix
            .iter()
            .flat_map(|s| &s.0)
            .chain(sc.prefix.iter().flat_map(|p| &p.0))
            .any(is_heredoc);
        Some(Self {
            name,
            args,
            has_heredoc,
        })
    }
}

/// Remove surrounding quotes from a string.
#[must_use]
pub fn unquote(s: &str) -> String {
    unquote_str(s)
}

/// Split an [`AndOr`] node into its [`Connector`] and pipeline.
fn split_and_or(ao: &AndOr) -> (Option<Connector>, &Pipeline) {
    match ao {
        AndOr::And(p) => (Some(Connector::And), p),
        AndOr::Or(p) => (Some(Connector::Or), p),
    }
}

/// Create a [`PipelineContext`] from a connector–pipeline pair.
///
/// - Return `None` if the pipeline contains no simple commands
fn pipeline_to_connector(
    (connector, pipeline): (Option<Connector>, &Pipeline),
) -> Option<PipelineContext> {
    let children = walk_pipeline(pipeline);
    (!children.is_empty()).then_some(PipelineContext {
        connector,
        children,
    })
}

/// Extract [`SimpleContext`] from each simple command in a pipeline.
fn walk_pipeline(pipeline: &Pipeline) -> Vec<SimpleContext> {
    pipeline
        .seq
        .iter()
        .filter_map(SimpleContext::from_command)
        .collect()
}

fn is_heredoc(item: &CommandPrefixOrSuffixItem) -> bool {
    matches!(
        item,
        CommandPrefixOrSuffixItem::IoRedirect(IoRedirect::HereDocument(..))
    )
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
        let p = CompleteContext::parse("git status").expect("should parse");
        assert_eq!(p.children.len(), 1);
        let cmd = &p.children[0].children[0];
        assert_eq!(cmd.name, "git");
        assert_eq!(cmd.args[0], "status");
    }

    #[test]
    fn parse_and_chain() {
        let p = CompleteContext::parse("ls && git status").expect("should parse");
        assert_eq!(p.children.len(), 2);
        assert_eq!(p.children[0].connector, None);
        assert_eq!(p.children[1].connector, Some(Connector::And));
        assert_eq!(p.children[1].children[0].name, "git");
    }

    #[test]
    fn parse_or_chain() {
        let p = CompleteContext::parse("false || git stash clear").expect("should parse");
        assert_eq!(p.children[1].connector, Some(Connector::Or));
        assert_eq!(p.children[1].children[0].name, "git");
    }

    #[test]
    fn parse_pipe() {
        let p = CompleteContext::parse("git log | head -5").expect("should parse");
        assert_eq!(p.children.len(), 1);
        assert_eq!(p.children[0].children.len(), 2);
        assert_eq!(p.children[0].children[0].name, "git");
        assert_eq!(p.children[0].children[1].name, "head");
    }

    #[test]
    fn parse_heredoc() {
        let p = CompleteContext::parse("cargo insta review <<EOF\na\nEOF").expect("should parse");
        assert!(p.children[0].children[0].has_heredoc);
    }

    #[test]
    fn parse_no_heredoc() {
        let p = CompleteContext::parse("cargo insta review").expect("should parse");
        assert!(!p.children[0].children[0].has_heredoc);
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
        let p = CompleteContext::parse("git commit -m 'msg'&& git push").expect("should parse");
        assert_eq!(p.children.len(), 2);
        assert_eq!(p.children[1].children[0].name, "git");
    }
}
