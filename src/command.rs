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
        let [cc] = program.complete_commands.as_slice() else {
            return None;
        };
        let [statement] = cc.0.as_slice() else {
            return None;
        };
        let aol = &statement.0;
        let mut children = Vec::new();
        for pair in
            std::iter::once((None, &aol.first)).chain(aol.additional.iter().map(split_and_or))
        {
            let pipeline_children = walk_pipeline(pair.1)?;
            if !pipeline_children.is_empty() {
                children.push(PipelineContext {
                    connector: pair.0,
                    children: pipeline_children,
                });
            }
        }
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
    /// Create a [`SimpleContext`] from a [`SimpleCommand`](SimpleCommand).
    ///
    /// - Extract the command name and positional arguments
    /// - Detect heredoc redirects in prefix and suffix
    fn from_simple_command(sc: &SimpleCommand) -> Option<Self> {
        let name = unquote(&sc.word_or_name.as_ref()?.value);
        let all_items = sc
            .suffix
            .iter()
            .flat_map(|s| &s.0)
            .chain(sc.prefix.iter().flat_map(|p| &p.0));
        let mut args = Vec::new();
        let mut has_heredoc = false;
        for item in all_items {
            match item {
                CommandPrefixOrSuffixItem::Word(w) => args.push(w.value.clone()),
                CommandPrefixOrSuffixItem::IoRedirect(IoRedirect::HereDocument(..)) => {
                    has_heredoc = true;
                }
                CommandPrefixOrSuffixItem::IoRedirect(r) if is_safe_redirect(r) => {}
                CommandPrefixOrSuffixItem::IoRedirect(_) => return None,
                _ => {}
            }
        }
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

/// Extract [`SimpleContext`] from each command in a pipeline.
///
/// Returns `None` if any command is compound (while, for, if, etc.)
/// so the evaluator falls through to Claude Code's default approval flow.
fn walk_pipeline(pipeline: &Pipeline) -> Option<Vec<SimpleContext>> {
    pipeline
        .seq
        .iter()
        .map(|cmd| match cmd {
            Command::Simple(sc) => SimpleContext::from_simple_command(sc),
            _ => None,
        })
        .collect()
}

/// A redirect is safe if it's an fd dup (`2>&1`) or targets `/dev/null`.
fn is_safe_redirect(r: &IoRedirect) -> bool {
    match r {
        IoRedirect::File(
            _,
            IoFileRedirectKind::DuplicateInput | IoFileRedirectKind::DuplicateOutput,
            IoFileRedirectTarget::Fd(_) | IoFileRedirectTarget::Duplicate(_),
        ) => true,
        IoRedirect::File(_, _, IoFileRedirectTarget::Filename(w)) => w.value == "/dev/null",
        _ => false,
    }
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

    #[test]
    fn while_loop_returns_none() {
        assert!(CompleteContext::parse("while true; do echo hi; done").is_none());
    }

    #[test]
    fn pipe_into_while_returns_none() {
        assert!(CompleteContext::parse("git log | while read line; do echo $line; done").is_none());
    }

    #[test]
    fn for_loop_returns_none() {
        assert!(CompleteContext::parse("for f in *.tmp; do echo $f; done").is_none());
    }

    #[test]
    fn if_then_returns_none() {
        assert!(CompleteContext::parse("if true; then echo hello; fi").is_none());
    }

    #[test]
    fn subshell_returns_none() {
        assert!(CompleteContext::parse("(echo hello && echo world)").is_none());
    }

    #[test]
    fn brace_group_returns_none() {
        assert!(CompleteContext::parse("{ echo hello; echo world; }").is_none());
    }

    #[test]
    fn chained_with_while_returns_none() {
        assert!(CompleteContext::parse("git status && while true; do echo hi; done").is_none());
    }

    #[test]
    fn semicolon_separated() {
        assert!(CompleteContext::parse("git status ; echo hi").is_none());
    }

    #[test]
    fn redirect_to_file_returns_none() {
        assert!(CompleteContext::parse("echo hi > /tmp/file").is_none());
    }

    #[test]
    fn redirect_append_returns_none() {
        assert!(CompleteContext::parse("echo hi >> /tmp/file").is_none());
    }

    #[test]
    fn redirect_overwrite_returns_none() {
        assert!(CompleteContext::parse("echo '' > ~/.ssh/authorized_keys").is_none());
    }

    #[test]
    fn redirect_dev_null_allowed() {
        let p = CompleteContext::parse("git status 2>/dev/null").expect("should parse");
        assert_eq!(p.children[0].children[0].name, "git");
    }

    #[test]
    fn redirect_fd_dup_allowed() {
        let p = CompleteContext::parse("cargo test 2>&1").expect("should parse");
        assert_eq!(p.children[0].children[0].name, "cargo");
    }

    #[test]
    fn redirect_input_returns_none() {
        assert!(CompleteContext::parse("cat < /etc/passwd").is_none());
    }

    #[test]
    fn redirect_clobber_returns_none() {
        assert!(CompleteContext::parse("echo hi >| /tmp/file").is_none());
    }
}
