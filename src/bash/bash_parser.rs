use crate::prelude::*;
use brush_parser::ast::*;
use brush_parser::word::{WordPiece, WordPieceWithSource};
use brush_parser::*;

/// Shell command parser that walks the brush-parser AST into [`CompleteContext`].
#[derive(Clone)]
pub struct BashParser {
    /// Compound structures the current parse position is nested inside.
    nesting: Vec<Nesting>,
}

impl BashParser {
    /// Create a new [`BashParser`] with default state.
    pub fn new() -> Self {
        Self {
            nesting: Vec::new(),
        }
    }

    /// Parse a shell command string into a [`CompleteContext`].
    ///
    /// - Returns `Err(ParseError::Skip)` for unsupported constructs so the
    ///   caller can fall through to the default approval flow
    /// - Returns `Err` for genuine parse failures (malformed syntax)
    pub fn parse(&mut self, command: &str) -> Result<CompleteContext, Report<ParseError>> {
        trace!(command, "Parsing");
        let tokens = tokenize_str(command).change_context(ParseError::Tokenize)?;
        let program =
            parse_tokens(&tokens, &ParserOptions::default()).change_context(ParseError::Tokens)?;
        let context = CompleteContext {
            raw: command.to_owned(),
            children: self.pipelines_from_program(&program)?,
        };
        trace!(pipelines = context.children.len(), "Parsed");
        Ok(context)
    }

    /// Extract pipelines from a parsed shell program.
    ///
    /// Returns `Err(SkipReason)` for unsupported constructs so the caller can
    /// fall through to the default approval flow.
    #[expect(
        clippy::indexing_slicing,
        reason = "length is exactly 1 after the guards above"
    )]
    fn pipelines_from_program(
        &mut self,
        program: &Program,
    ) -> Result<Vec<PipelineContext>, Report<ParseError>> {
        if program.complete_commands.is_empty() {
            return Err(ParseError::skip(SkipReason::ZeroCommands));
        } else if program.complete_commands.len() > 1 {
            return Err(ParseError::skip(SkipReason::MultipleCommands));
        }
        let statements = &program.complete_commands[0].0;
        if statements.is_empty() {
            return Err(ParseError::skip(SkipReason::ZeroStatements));
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
    ) -> Result<Vec<PipelineContext>, Report<ParseError>> {
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
    /// - Pushes [`Nesting::For`] and extracts body commands for `for` loops
    /// - Returns `Err` for unsupported compound commands
    fn pipeline_to_commands(
        &mut self,
        pipeline: &Pipeline,
    ) -> Result<Vec<SimpleContext>, Report<ParseError>> {
        let mut commands = Vec::new();
        for command in &pipeline.seq {
            match command {
                Command::Simple(simple) => {
                    commands.extend(self.contexts_from_simple_command(simple)?);
                }
                Command::Compound(CompoundCommand::ForClause(f), redirects) => {
                    commands.extend(self.contexts_from_for(f, redirects.as_ref())?);
                }
                _ => return Err(ParseError::skip(SkipReason::UnsupportedCompound)),
            }
        }
        Ok(commands)
    }

    /// Create [`SimpleContext`] from a [`ForClauseCommand`].
    ///
    /// Rejects nested for loops, redirects on the loop, and substitutions
    /// in the iteration values.
    fn contexts_from_for(
        &mut self,
        for_clause: &ForClauseCommand,
        redirects: Option<&RedirectList>,
    ) -> Result<Vec<SimpleContext>, Report<ParseError>> {
        if redirects.is_some() {
            return Err(ParseError::skip(SkipReason::ForLoopRedirect));
        }
        if self.nesting.contains(&Nesting::For) {
            return Err(ParseError::skip(SkipReason::NestedForLoop));
        }
        if let Some(values) = &for_clause.values {
            for word in values {
                let subs = extract_substitutions(&word.value)?;
                if !subs.is_empty() {
                    return Err(ParseError::skip(SkipReason::ForLoopSubstitution));
                }
            }
        }
        self.nesting.push(Nesting::For);
        let mut contexts = Vec::new();
        for item in &for_clause.body.list.0 {
            for p in self.aol_to_pipelines(&item.0, None)? {
                contexts.extend(p.children);
            }
        }
        self.nesting.pop();
        trace!(body_commands = contexts.len(), "Parsed for loop");
        Ok(contexts)
    }

    /// Create [`SimpleContext`] from a [`SimpleCommand`].
    ///
    /// - Extracts the command name and positional arguments
    /// - Detects heredoc redirects in prefix and suffix
    /// - Detects command substitutions in arguments structurally
    /// - Parses inner substitution commands recursively, one level only
    /// - Returns the outer command first, followed by any inner commands
    fn contexts_from_simple_command(
        &mut self,
        simple: &SimpleCommand,
    ) -> Result<Vec<SimpleContext>, Report<ParseError>> {
        let word = simple
            .word_or_name
            .as_ref()
            .ok_or_else(|| ParseError::skip(SkipReason::BareAssignment))?;
        let name_subs = extract_substitutions(&word.value)?;
        if !name_subs.is_empty() {
            return Err(ParseError::skip(SkipReason::CommandNameSubstitution));
        }
        let name = unquote_str(&word.value);
        let all_items = simple
            .suffix
            .iter()
            .flat_map(|suffix| &suffix.0)
            .chain(simple.prefix.iter().flat_map(|p| &p.0));
        let mut args = Vec::new();
        let mut has_heredoc = false;
        let mut contains_substitution = false;
        let mut inner_commands = Vec::new();
        for item in all_items {
            match item {
                CommandPrefixOrSuffixItem::Word(w) => {
                    let subs = extract_substitutions(&w.value)?;
                    if !subs.is_empty() {
                        contains_substitution = true;
                        for sub in &subs {
                            inner_commands.extend(self.parse_substitution(sub)?);
                        }
                    }
                    args.push(w.value.clone());
                }
                CommandPrefixOrSuffixItem::IoRedirect(IoRedirect::HereDocument(..)) => {
                    has_heredoc = true;
                }
                CommandPrefixOrSuffixItem::IoRedirect(r) if is_safe_redirect(r) => {}
                CommandPrefixOrSuffixItem::IoRedirect(_) => {
                    return Err(ParseError::skip(SkipReason::UnsafeRedirect));
                }
                CommandPrefixOrSuffixItem::ProcessSubstitution(..) => {
                    return Err(ParseError::skip(SkipReason::ProcessSubstitution));
                }
                CommandPrefixOrSuffixItem::AssignmentWord(..) => {}
            }
        }
        trace!(
            %name,
            args = args.len(),
            has_heredoc,
            contains_substitution,
            inner_commands = inner_commands.len(),
            "Parsed simple command",
        );
        let mut result = vec![SimpleContext {
            name,
            args,
            has_heredoc,
            contains_substitution,
            nesting: self.nesting.clone(),
        }];
        result.extend(inner_commands);
        Ok(result)
    }

    /// Parse a command substitution string one level deep.
    ///
    /// - Clones the parser and pushes [`Nesting::Substitution`]
    /// - Returns `Err(NestedSubstitution)` if already inside a substitution
    /// - Bubbles up any inner failure since unparseable substitutions cannot
    ///   be reasoned about
    fn parse_substitution(&self, command: &str) -> Result<Vec<SimpleContext>, Report<ParseError>> {
        trace!(command, "Parsing substitution");
        if self.nesting.contains(&Nesting::Substitution) {
            return Err(ParseError::skip(SkipReason::NestedSubstitution));
        }
        let mut inner = self.clone();
        inner.nesting.push(Nesting::Substitution);
        let ctx = inner.parse(command)?;
        Ok(ctx.children.into_iter().flat_map(|p| p.children).collect())
    }
}

/// Extract command substitution strings from a shell word.
fn extract_substitutions(word: &str) -> Result<Vec<String>, Report<ParseError>> {
    let pieces = word::parse(word, &ParserOptions::default()).change_context(ParseError::Word)?;
    let mut subs = Vec::new();
    collect_substitutions(&pieces, &mut subs)?;
    Ok(subs)
}

/// Recursively collect command substitution strings from parsed word pieces.
///
/// Returns `Err(ParameterSubstitution)` if a parameter expansion could
/// contain substitutions in its opaque string fields.
fn collect_substitutions(
    pieces: &[WordPieceWithSource],
    out: &mut Vec<String>,
) -> Result<(), Report<ParseError>> {
    for piece in pieces {
        match &piece.piece {
            WordPiece::CommandSubstitution(s) | WordPiece::BackquotedCommandSubstitution(s) => {
                out.push(s.clone());
            }
            WordPiece::DoubleQuotedSequence(inner)
            | WordPiece::GettextDoubleQuotedSequence(inner) => {
                collect_substitutions(inner, out)?;
            }
            WordPiece::ArithmeticExpression(_) => {
                return Err(ParseError::skip(SkipReason::ArithmeticSubstitution));
            }
            WordPiece::ParameterExpansion(
                word::ParameterExpr::Parameter { .. } | word::ParameterExpr::ParameterLength { .. },
            )
            | WordPiece::Text(_)
            | WordPiece::SingleQuotedText(_)
            | WordPiece::AnsiCQuotedText(_)
            | WordPiece::TildeExpansion(_)
            | WordPiece::EscapeSequence(_) => {}
            WordPiece::ParameterExpansion(_) => {
                return Err(ParseError::skip(SkipReason::ParameterSubstitution));
            }
        }
    }
    Ok(())
}

/// Errors returned by [`BashParser`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ParseError {
    /// Shell tokenization failed, such as unmatched quotes.
    #[error("Failed to tokenize the command")]
    Tokenize,
    /// Token stream could not be parsed into a shell AST.
    #[error("Failed to parse tokens")]
    Tokens,
    /// Word-level parse failed for substitution extraction.
    #[error("Failed to parse a word")]
    Word,
    /// Command was parsed successfully but skipped.
    #[error("Skipped: {0}")]
    Skip(SkipReason),
}

impl ParseError {
    /// Wrap a [`SkipReason`] in a [`Report`] for early return.
    pub(crate) fn skip(reason: SkipReason) -> Report<Self> {
        Report::new(Self::Skip(reason))
    }
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
    BashParser::new()
        .parse(command)
        .expect("command should parse")
}

#[cfg(test)]
/// Parse `command`, expecting a [`SkipReason`].
#[expect(clippy::panic, reason = "test helper")]
pub(crate) fn parse_expect_skip(command: &str) -> SkipReason {
    match BashParser::new()
        .parse(command)
        .expect_err("command should not succeed")
        .current_context()
    {
        ParseError::Skip(reason) => *reason,
        other => panic!("expected Skip, got {other:?}"),
    }
}

#[cfg(test)]
/// Parse `command`, expecting a [`ParseError`] that is not a skip.
pub(crate) fn parse_expect_error(command: &str) -> ParseError {
    let report = BashParser::new()
        .parse(command)
        .expect_err("command should fail to parse");
    eprintln!("{report:?}");
    let error = *report.current_context();
    assert!(
        !matches!(error, ParseError::Skip(_)),
        "expected a real error, got {error:?}"
    );
    error
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
    fn for_loop_substitution() {
        let reason = parse_expect_skip("for f in $(find . -name '*.rs'); do echo $f; done");
        assert_eq!(reason, SkipReason::ForLoopSubstitution);
    }

    #[test]
    fn for_loop_backtick_substitution() {
        let reason = parse_expect_skip("for f in `ls`; do echo $f; done");
        assert_eq!(reason, SkipReason::ForLoopSubstitution);
    }

    #[test]
    fn command_substitution_dollar() {
        let context = parse_expect_context("echo $(whoami)");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn command_substitution_backtick() {
        let context = parse_expect_context("echo `whoami`");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn command_substitution_in_arg() {
        let context = parse_expect_context("git commit -m \"$(date)\"");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn command_substitution_as_name() {
        let reason = parse_expect_skip("$(whoami)");
        assert_eq!(reason, SkipReason::CommandNameSubstitution);
    }

    #[test]
    fn command_substitution_backtick_as_name() {
        let reason = parse_expect_skip("`whoami`");
        assert_eq!(reason, SkipReason::CommandNameSubstitution);
    }

    #[test]
    fn nested_command_substitution() {
        let reason = parse_expect_skip("echo $(echo $(whoami))");
        assert_eq!(reason, SkipReason::NestedSubstitution);
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

    #[test]
    fn substitution_inner_parse_error() {
        let error = parse_expect_error("echo $(;;)");
        assert_eq!(error, ParseError::Tokens);
    }

    #[test]
    fn substitution_inner_bare_semicolon() {
        let error = parse_expect_error("echo $(;)");
        assert_eq!(error, ParseError::Tokens);
    }

    #[test]
    fn substitution_inner_if() {
        let reason = parse_expect_skip("echo $(if true; then echo x; fi)");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn substitution_inner_while() {
        let reason = parse_expect_skip("echo $(while true; do echo x; done)");
        assert_eq!(reason, SkipReason::UnsupportedCompound);
    }

    #[test]
    fn substitution_inner_empty() {
        let reason = parse_expect_skip("echo $()");
        assert_eq!(reason, SkipReason::ZeroCommands);
    }

    #[test]
    fn substitution_inner_bare_redirect() {
        let reason = parse_expect_skip("echo $(>&2)");
        assert_eq!(reason, SkipReason::BareAssignment);
    }

    #[test]
    fn substitution_inner_for_loop() {
        let context = parse_expect_context("echo $(for x in a b; do echo $x; done)");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn multiple_substitutions_in_one_command() {
        let context = parse_expect_context("echo $(whoami) $(date)");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn single_quoted_substitution_not_expanded() {
        let context = parse_expect_context("echo '$(whoami)'");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn substitution_in_for_loop_body() {
        let context = parse_expect_context("for f in a b; do echo $(whoami); done");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn substitution_in_piped_command() {
        let context = parse_expect_context("echo $(whoami) | head -1");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn substitution_in_chained_command() {
        let context = parse_expect_context("echo $(whoami) && git status");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn inner_command_with_args() {
        let context = parse_expect_context("git commit -m \"$(date +%Y)\"");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn substitution_embedded_in_word() {
        let context = parse_expect_context("echo file-$(date).txt");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn backtick_inside_dollar_substitution() {
        let reason = parse_expect_skip("echo $(echo `whoami`)");
        assert_eq!(reason, SkipReason::NestedSubstitution);
    }

    #[test]
    fn parameter_expansion_with_default_substitution() {
        let reason = parse_expect_skip("echo ${var:-$(whoami)}");
        assert_eq!(reason, SkipReason::ParameterSubstitution);
    }

    #[test]
    fn parameter_expansion_simple() {
        let context = parse_expect_context("echo $var");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parameter_expansion_braced() {
        let context = parse_expect_context("echo ${var}");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parameter_expansion_length() {
        let context = parse_expect_context("echo ${#var}");
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn parameter_expansion_alternative() {
        let reason = parse_expect_skip("echo ${var:+replacement}");
        assert_eq!(reason, SkipReason::ParameterSubstitution);
    }

    #[test]
    fn parameter_expansion_suffix_removal() {
        let reason = parse_expect_skip("echo ${var%%.tmp}");
        assert_eq!(reason, SkipReason::ParameterSubstitution);
    }

    #[test]
    fn complex_multi_statement_with_substitutions_and_redirects() {
        let cmd = r#"cargo doc --document-private-items -p globset 2>&1 | tail -5; grep -r "impl.*Debug" $(find target/doc -name "struct.GlobMatcher.html" 2>/dev/null) 2>/dev/null || echo "checking source instead"; grep "derive" $(find . -path "*/globset/src/*.rs" -not -path "./target/*" 2>/dev/null) 2>/dev/null || echo "not in local source""#;
        let context = parse_expect_context(cmd);
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn process_substitution_input() {
        let reason = parse_expect_skip("diff <(echo a) <(echo b)");
        assert_eq!(reason, SkipReason::ProcessSubstitution);
    }

    #[test]
    fn process_substitution_output() {
        let reason = parse_expect_skip("tee >(grep error)");
        assert_eq!(reason, SkipReason::ProcessSubstitution);
    }

    #[test]
    fn arithmetic_expression() {
        let reason = parse_expect_skip("echo $((1 + 2))");
        assert_eq!(reason, SkipReason::ArithmeticSubstitution);
    }

    /// Known issue: <https://github.com/reubeno/brush/pull/420>
    /// Resolved in: <https://github.com/reubeno/brush/pull/1067>
    #[test]
    fn command_sub_heredoc_unbalanced_single_quote() {
        let context = parse_expect_context("echo \"$(cat <<'EOF'\nit's\nEOF\n)\"");
        assert_yaml_snapshot!(context);
    }

    /// Known issue: <https://github.com/reubeno/brush/issues/1066>
    #[test]
    fn command_sub_heredoc_unbalanced_double_quote() {
        let error = parse_expect_error("echo \"$(cat <<'EOF'\nit\"s\nEOF\n)\"");
        assert_eq!(error, ParseError::Word);
    }

    #[test]
    fn heredoc_quoted_tag_in_chained_command() {
        let cmd =
            "git add file.md && git commit -m \"$(cat <<'EOF'\nfeat(scope): Add feature\nEOF\n)\"";
        let context = parse_expect_context(cmd);
        assert_yaml_snapshot!(context);
    }

    #[test]
    fn text_pieces_never_contain_substitutions() {
        let inputs = [
            "hello$(whoami)world",
            "$(a)$(b)",
            "prefix$(cmd)",
            "$(cmd)suffix",
            "`cmd`rest",
            "a`b`c$(d)e",
        ];
        for input in inputs {
            let pieces = word::parse(input, &ParserOptions::default()).expect("word should parse");

            for p in &pieces {
                if let WordPiece::Text(s)
                | WordPiece::SingleQuotedText(s)
                | WordPiece::AnsiCQuotedText(s)
                | WordPiece::EscapeSequence(s) = &p.piece
                {
                    assert!(
                        !s.contains("$(") && !s.contains('`'),
                        "leaf piece {s:?} in {input:?} contains substitution syntax"
                    );
                }
            }
        }
    }
}
