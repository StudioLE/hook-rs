use crate::prelude::*;
use brush_parser::ast::*;
use brush_parser::*;

/// Shell command parser that walks the brush-parser AST into [`CompleteContext`].
pub struct Parser {
    has_for_loop: bool,
}

impl Parser {
    /// Create a new [`Parser`] with default state.
    pub fn new() -> Self {
        Self {
            has_for_loop: false,
        }
    }

    /// Parse a shell command string into a [`CompleteContext`].
    ///
    /// Returns `Ok(None)` for unsupported constructs (compound commands, unsafe
    /// redirects) so the caller can fall through to the default approval flow.
    /// Returns `Err` for genuine parse failures (malformed syntax).
    pub fn parse_str(
        &mut self,
        command: &str,
    ) -> Result<Result<CompleteContext, SkipReason>, Report<ParseError>> {
        if command.contains("$(") || command.contains('`') {
            return Ok(Err(SkipReason::CommandSubstitution));
        }
        let tokens = tokenize_str(command).change_context(ParseError::Tokenize)?;
        let program = parse_tokens(&tokens, &ParserOptions::default(), &SourceInfo::default())
            .change_context(ParseError::ParseTokens)?;
        match self.from_program(&program) {
            Ok(children) => Ok(Ok(CompleteContext {
                raw: command.to_owned(),
                children,
                has_for: self.has_for_loop,
            })),
            Err(reason) => Ok(Err(reason)),
        }
    }

    /// Extract pipelines from a parsed shell program.
    ///
    /// Returns `Err(SkipReason)` for unsupported constructs so the caller can
    /// fall through to the default approval flow.
    #[expect(
        clippy::indexing_slicing,
        reason = "length is exactly 1 after the guards above"
    )]
    fn from_program(&mut self, program: &Program) -> Result<Vec<PipelineContext>, SkipReason> {
        if program.complete_commands.is_empty() {
            return Err(SkipReason::ZeroCommands);
        } else if program.complete_commands.len() > 1 {
            return Err(SkipReason::MultipleCommands);
        }
        let statements = &program.complete_commands[0].0;
        if statements.is_empty() {
            return Err(SkipReason::ZeroStatements);
        }
        let mut children = Vec::new();
        for aol in statements.iter().map(|s| &s.0) {
            let connector = if children.is_empty() {
                None
            } else {
                Some(Connector::Semi)
            };
            children.extend(self.aol_to_pipelines(aol, connector)?);
        }
        Ok(children)
    }

    fn aol_to_pipelines(
        &mut self,
        aol: &AndOrList,
        first_connector: Option<Connector>,
    ) -> Result<Vec<PipelineContext>, SkipReason> {
        let mut pipelines = Vec::new();
        pipelines.push(PipelineContext {
            children: self.pipeline_to_commands(&aol.first)?,
            connector: first_connector,
        });
        for and_or in &aol.additional {
            let (connector, pipeline) = match and_or {
                AndOr::And(p) => (Connector::And, p),
                AndOr::Or(p) => (Connector::Or, p),
            };
            pipelines.push(PipelineContext {
                children: self.pipeline_to_commands(pipeline)?,
                connector: Some(connector),
            });
        }
        Ok(pipelines)
    }

    /// Extract [`SimpleContext`] from each command in a pipeline.
    ///
    /// For `for` loops, extracts body commands and sets `has_for_loop`.
    /// Returns `Err` for unsupported compound commands (while, if, etc.).
    fn pipeline_to_commands(
        &mut self,
        pipeline: &Pipeline,
    ) -> Result<Vec<SimpleContext>, SkipReason> {
        let mut commands = Vec::new();
        for command in &pipeline.seq {
            match command {
                Command::Simple(simple) => {
                    commands.push(SimpleContext::from_simple_command(simple)?);
                }
                Command::Compound(CompoundCommand::ForClause(f), redirects) => {
                    if redirects.is_some() {
                        return Err(SkipReason::ForLoopRedirect);
                    }
                    if self.has_for_loop {
                        return Err(SkipReason::NestedForLoop);
                    }
                    self.has_for_loop = true;
                    for item in &f.body.list.0 {
                        for p in self.aol_to_pipelines(&item.0, None)? {
                            commands.extend(p.children);
                        }
                    }
                }
                _ => return Err(SkipReason::UnsupportedCompound),
            }
        }
        Ok(commands)
    }
}

impl SimpleContext {
    /// Create a [`SimpleContext`] from a [`SimpleCommand`](SimpleCommand).
    ///
    /// - Extract the command name and positional arguments
    /// - Detect heredoc redirects in prefix and suffix
    fn from_simple_command(simple: &SimpleCommand) -> Result<Self, SkipReason> {
        let word = simple
            .word_or_name
            .as_ref()
            .ok_or(SkipReason::BareAssignment)?;
        let name = unquote_str(&word.value);
        let all_items = simple
            .suffix
            .iter()
            .flat_map(|suffix| &suffix.0)
            .chain(simple.prefix.iter().flat_map(|p| &p.0));
        let mut args = Vec::new();
        let mut has_heredoc = false;
        for item in all_items {
            match item {
                CommandPrefixOrSuffixItem::Word(w) => args.push(w.value.clone()),
                CommandPrefixOrSuffixItem::IoRedirect(IoRedirect::HereDocument(..)) => {
                    has_heredoc = true;
                }
                CommandPrefixOrSuffixItem::IoRedirect(r) if is_safe_redirect(r) => {}
                CommandPrefixOrSuffixItem::IoRedirect(_) => {
                    return Err(SkipReason::UnsafeRedirect);
                }
                _ => {}
            }
        }
        Ok(Self {
            name,
            args,
            has_heredoc,
        })
    }
}

/// Errors from shell command parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ParseError {
    /// Shell tokenization failed, such as unmatched quotes.
    #[error("tokenize command")]
    Tokenize,
    /// Token stream could not be parsed into a shell AST.
    #[error("parse tokens")]
    ParseTokens,
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
/// Parse `command`, expecting a successful [`CompleteContext`].
pub(crate) fn parse_expect_context(command: &str) -> CompleteContext {
    Parser::new()
        .parse_str(command)
        .expect("command should be parseable")
        .expect("command should not be skipped")
}

#[cfg(test)]
/// Parse `command`, expecting a [`SkipReason`].
pub(crate) fn parse_expect_skip(command: &str) -> SkipReason {
    Parser::new()
        .parse_str(command)
        .expect("command should be parseable")
        .expect_err("command should be skipped")
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn parse_simple() {
        let context = parse_expect_context("git status");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parse_and_chain() {
        let context = parse_expect_context("ls && git status");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parse_or_chain() {
        let context = parse_expect_context("false || git stash clear");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parse_pipe() {
        let context = parse_expect_context("git log | head -5");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parse_heredoc() {
        let context = parse_expect_context("cargo insta review <<EOF\na\nEOF");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parse_no_heredoc() {
        let context = parse_expect_context("cargo insta review");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn unquote_single() {
        assert_eq!(unquote_str("'hello'"), "hello");
    }

    #[test]
    fn unquote_double() {
        assert_eq!(unquote_str("\"hello\""), "hello");
    }

    #[test]
    fn unquote_bare() {
        assert_eq!(unquote_str("hello"), "hello");
    }

    #[test]
    fn no_space_before_and() {
        let context = parse_expect_context("git commit -m 'msg'&& git push");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn while_loop() {
        let reason = parse_expect_skip("while true; do echo hi; done");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn pipe_into_while() {
        let reason = parse_expect_skip("git log | while true; do echo hi; done");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn for_loop() {
        let context = parse_expect_context("for f in *.tmp; do echo $f; done");
        assert!(context.has_for);
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn nested_for_loop() {
        let reason = parse_expect_skip("for f in *.tmp; do for g in *.log; do echo $g; done; done");
        assert_eq!(reason, SkipReason::NestedForLoop);
    }

    #[test]
    fn for_loop_redirect() {
        let reason = parse_expect_skip("for f in *.tmp; do echo $f; done > /tmp/out");
        assert_eq!(reason, SkipReason::ForLoopRedirect);
    }

    #[test]
    fn command_substitution_dollar() {
        let reason = parse_expect_skip("echo $(whoami)");
        assert_eq!(reason, SkipReason::CommandSubstitution);
    }

    #[test]
    fn command_substitution_backtick() {
        let reason = parse_expect_skip("echo `whoami`");
        assert_eq!(reason, SkipReason::CommandSubstitution);
    }

    #[test]
    fn command_substitution_in_arg() {
        let reason = parse_expect_skip("git commit -m \"$(date)\"");
        assert_eq!(reason, SkipReason::CommandSubstitution);
    }

    #[test]
    fn if_then() {
        let reason = parse_expect_skip("if true; then echo hello; fi");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn subshell() {
        let reason = parse_expect_skip("(echo hello && echo world)");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn brace_group() {
        let reason = parse_expect_skip("{ echo hello; echo world; }");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn chained_with_while() {
        let reason = parse_expect_skip("git status && while true; do echo hi; done");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn semicolon_separated() {
        let context = parse_expect_context("git status ; echo hi");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn redirect_to_file() {
        let reason = parse_expect_skip("echo hi > /tmp/file");
        assert_eq!(reason, SkipReason::UnsafeRedirect);
    }

    #[test]
    fn redirect_append() {
        let reason = parse_expect_skip("echo hi >> /tmp/file");
        assert_eq!(reason, SkipReason::UnsafeRedirect);
    }

    #[test]
    fn redirect_overwrite() {
        let reason = parse_expect_skip("echo '' > ~/.ssh/authorized_keys");
        assert_eq!(reason, SkipReason::UnsafeRedirect);
    }

    #[test]
    fn redirect_dev_null() {
        let context = parse_expect_context("git status 2>/dev/null");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn redirect_fd_dup() {
        let context = parse_expect_context("cargo test 2>&1");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn redirect_input() {
        let reason = parse_expect_skip("cat < /etc/passwd");
        assert_eq!(reason, SkipReason::UnsafeRedirect);
    }

    #[test]
    fn redirect_clobber() {
        let reason = parse_expect_skip("echo hi >| /tmp/file");
        assert_eq!(reason, SkipReason::UnsafeRedirect);
    }
}
