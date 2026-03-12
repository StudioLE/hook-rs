//! Argument matcher for [`SimpleRule`] fields.

use globset::{GlobBuilder, GlobMatcher};

/// Argument matcher for [`SimpleRule`] fields.
///
/// Matches a command-line argument by pattern, optionally checking its
/// option-argument (the value that follows the flag).
///
/// - `Arg::new("x")` checks whether the flag/value is present in the args.
/// - `Arg::new("x").value("y")` checks whether the flag is present and its
///   option-argument matches the pattern. Three forms are recognized:
///   - Two-arg: `-X POST` (flag and value as separate args)
///   - Concatenated short: `-XPOST` (short flags only)
///   - Equals long: `--data=foo` (long flags only)
/// - Value matching is case-sensitive by default; use `.ivalue()` for case-insensitive.
pub struct Arg {
    /// Raw option pattern string.
    option: String,
    /// Pre-compiled glob for `option`, if it contains glob chars.
    option_glob: Option<GlobMatcher>,
    /// Raw value pattern string.
    value: Option<String>,
    /// Pre-compiled glob for `value`, if it contains glob chars.
    value_glob: Option<GlobMatcher>,
    /// Match the value pattern case-insensitively.
    case_insensitive: bool,
}

impl Arg {
    /// Create a new [`Arg`] matching the given option pattern.
    pub fn new(option: impl Into<String>) -> Self {
        let option = option.into();
        let option_glob = compile_glob(&option, false);
        Self {
            option,
            option_glob,
            value: None,
            value_glob: None,
            case_insensitive: false,
        }
    }

    /// Also check the option-argument against this pattern.
    ///
    /// Matching is case-sensitive.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.value_glob = compile_glob(&value, false);
        self.value = Some(value);
        self.case_insensitive = false;
        self
    }

    /// Also check the option-argument against this pattern.
    ///
    /// Matching is case-insensitive.
    pub fn ivalue(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.value_glob = compile_glob(&value, true);
        self.value = Some(value);
        self.case_insensitive = true;
        self
    }

    /// Check if this argument is present in `args`.
    ///
    /// When `value` is `None`, checks whether `option` matches any element.
    /// When `value` is `Some(pattern)`, finds `option` and checks the
    /// option-argument against `pattern` in three forms:
    /// - Two-arg: the next element (e.g. `-X POST`)
    /// - Concatenated short: the remainder after the flag char (e.g. `-XPOST`, short flags only)
    /// - Equals long: the portion after `=` (e.g. `--data=foo`, long flags only)
    pub(crate) fn is_present(&self, args: &[&str]) -> bool {
        if self.value.is_none() {
            return args.iter().any(|a| self.is_option_match(a));
        }
        for (i, arg) in args.iter().enumerate() {
            if let Some(next) = args.get(i + 1)
                && (self.is_option_match(arg) && self.is_value_match(next))
            {
                return true;
            }
            if self.is_concat_short_value_match(arg) || self.is_concat_long_value_match(arg) {
                return true;
            }
        }
        false
    }

    /// Check if a single arg matches the option pattern.
    fn is_option_match(&self, arg: &str) -> bool {
        self.is_glob_match(arg)
            || self.is_short_flag_match(arg)
            || self.is_long_flag_match(arg)
            || arg == self.option
    }

    /// Check if `arg` matches the option as a glob pattern.
    ///
    /// Only applies when the option contains glob metacharacters (`*`, `?`,
    /// `{`, or `[`), in which case a pre-compiled `globset` matcher is used.
    fn is_glob_match(&self, arg: &str) -> bool {
        let Some(glob) = &self.option_glob else {
            return false;
        };
        glob.is_match(arg)
    }

    /// Check if `arg` matches the option as a short flag.
    ///
    /// Short flags are a single hyphen + one ASCII char (e.g. `-d`, `-F`).
    /// - Matches standalone (`-d`)
    /// - Matches inside bundled short flags (`-fd`, `-fxd`)
    /// - Does NOT match long flags (`--debug`)
    ///
    /// **POSIX caveat:** in `-fd`, POSIX allows `-f` to consume `d` as its
    /// option-argument rather than `-d` being a separate flag.
    /// - We cannot distinguish these cases without per-command option definitions.
    /// - We assume all characters are independent flags.
    /// - See [`is_concat_short_value_match`](Self::is_concat_short_value_match)
    ///   for the full rationale.
    fn is_short_flag_match(&self, arg: &str) -> bool {
        if !arg.starts_with('-') || arg.starts_with("--") || arg.len() < 2 {
            return false;
        }
        let Some(flag) = self.get_short_flag() else {
            return false;
        };
        arg.chars().skip(1).any(|c| c == flag)
    }

    /// Check if `arg` matches the option as a long flag.
    ///
    /// Long flags (`--data`, `--field`) use exact string match and also match
    /// with an `=` suffix (e.g. `--data` matches `--data=foo`).
    fn is_long_flag_match(&self, arg: &str) -> bool {
        if !self.option.starts_with("--") {
            return false;
        }
        arg == self.option || arg.starts_with(&format!("{}=", self.option))
    }

    /// Flag character when `option` is a single-char short flag.
    ///
    /// Returns `Some('X')` for `-X`, `None` otherwise.
    fn get_short_flag(&self) -> Option<char> {
        let is_option_short =
            self.option.len() == 2 && self.option.starts_with('-') && self.option != "--";
        if is_option_short {
            self.option.chars().nth(1)
        } else {
            None
        }
    }

    /// Check if `value` matches the value pattern.
    ///
    /// Uses the pre-compiled glob matcher when available, otherwise falls back
    /// to literal comparison (case-insensitive when configured via `ivalue`).
    fn is_value_match(&self, value: &str) -> bool {
        if let Some(glob) = &self.value_glob {
            return glob.is_match(value);
        }
        let Some(v) = self.value.as_deref() else {
            return false;
        };
        if self.case_insensitive {
            value.eq_ignore_ascii_case(v)
        } else {
            value == v
        }
    }

    /// Check if `arg` is a concatenated short flag whose value matches the pattern.
    ///
    /// Only applies when `option` is a single-char short flag and `arg` starts
    /// with that flag char at position 1. For example, `-X` matches `-XPOST`
    /// but not `-aXPOST`.
    ///
    /// **POSIX deviation:** POSIX Utility Syntax Guideline 5 allows the
    /// value-taking flag anywhere in a bundle (e.g. `-aXPOST` means
    /// `-a -X POST`).
    /// - Resolving this requires per-command option definitions, which we
    ///   don't have.
    /// - We restrict to position 1 to avoid ambiguity.
    /// - This is acceptable because we evaluate commands generated by
    ///   Claude, not adversarial input. Claude should not be crafting
    ///   arguments to circumvent these rules.
    fn is_concat_short_value_match(&self, arg: &str) -> bool {
        if !arg.starts_with('-') || arg.starts_with("--") || arg.len() < 3 {
            return false;
        }
        let option = arg.chars().nth(1);
        if self.get_short_flag() != option {
            return false;
        }
        let value = &arg[2..];
        self.is_value_match(value)
    }

    /// Check if `arg` is a long flag with `=` whose value matches the pattern.
    ///
    /// Only applies when `option` is a long flag. Strips the `--flag=` prefix
    /// and checks the remainder against the value pattern.
    fn is_concat_long_value_match(&self, arg: &str) -> bool {
        if !self.option.starts_with("--") {
            return false;
        }
        let prefix = format!("{}=", self.option);
        let Some(value) = arg.strip_prefix(&prefix) else {
            return false;
        };
        self.is_value_match(value)
    }
}

/// Compile a glob pattern if the string contains glob metacharacters.
fn compile_glob(pattern: &str, case_insensitive: bool) -> Option<GlobMatcher> {
    let is_glob = pattern.contains('*')
        || pattern.contains('?')
        || pattern.contains('{')
        || pattern.contains('[');
    if !is_glob {
        return None;
    }
    GlobBuilder::new(pattern)
        .literal_separator(false)
        .case_insensitive(case_insensitive)
        .build()
        .ok()
        .map(|g| g.compile_matcher())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Arg::new - short flag bundling ===

    #[test]
    fn short_flag_standalone() {
        assert!(Arg::new("-d").is_present(&["-d"]));
    }

    #[test]
    fn short_flag_bundled_fd() {
        assert!(Arg::new("-d").is_present(&["-fd"]));
    }

    #[test]
    fn short_flag_bundled_fxd() {
        assert!(Arg::new("-d").is_present(&["-fxd"]));
    }

    #[test]
    fn short_flag_no_match_long() {
        assert!(!Arg::new("-d").is_present(&["--debug"]));
    }

    // === Arg::new - long flags (with = expansion) ===

    #[test]
    fn long_flag_exact() {
        assert!(Arg::new("--data").is_present(&["--data"]));
    }

    #[test]
    fn long_flag_equals() {
        assert!(Arg::new("--data").is_present(&["--data=foo"]));
    }

    #[test]
    fn bare_value_exact() {
        assert!(Arg::new("reset").is_present(&["reset"]));
    }

    #[test]
    fn bare_value_no_match() {
        assert!(!Arg::new("reset").is_present(&["status"]));
    }

    // === Arg::new - glob ===

    #[test]
    fn glob_tmp_match() {
        assert!(Arg::new("/tmp/*").is_present(&["/tmp/file.txt"]));
    }

    #[test]
    fn glob_tmp_no_match() {
        assert!(!Arg::new("/tmp/*").is_present(&["/var/file.txt"]));
    }

    // === Arg::new(...).value(...) - two-arg form ===

    #[test]
    fn value_two_arg_post() {
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(arg.is_present(&["-X", "POST"]));
    }

    #[test]
    fn value_two_arg_case_sensitive() {
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(!arg.is_present(&["-X", "post"]));
    }

    #[test]
    fn value_two_arg_case_insensitive() {
        let arg = Arg::new("-X").ivalue("{POST,PUT,PATCH,DELETE}");
        assert!(arg.is_present(&["-X", "post"]));
    }

    #[test]
    fn value_two_arg_no_match_get() {
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(!arg.is_present(&["-X", "GET"]));
    }

    #[test]
    fn value_two_arg_exec_rm() {
        let arg = Arg::new("-exec").value("rm");
        assert!(arg.is_present(&["-exec", "rm"]));
    }

    // === Arg::new(...).value(...) - concatenated short form ===

    #[test]
    fn value_concat_short_xpost() {
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(arg.is_present(&["-XPOST"]));
    }

    #[test]
    fn value_concat_short_case_insensitive() {
        let arg = Arg::new("-X").ivalue("{POST,PUT,PATCH,DELETE}");
        assert!(arg.is_present(&["-Xpost"]));
    }

    #[test]
    fn value_concat_short_glob() {
        let arg = Arg::new("-o").value("*.txt");
        assert!(arg.is_present(&["-ofile.txt"]));
    }

    // === Arg::new(...).value(...) - = long form ===

    #[test]
    fn value_equals_long_wildcard() {
        let arg = Arg::new("--data").value("*");
        assert!(arg.is_present(&["--data=foo"]));
    }

    #[test]
    fn value_equals_long_json() {
        let arg = Arg::new("--data").value("*.json");
        assert!(arg.is_present(&["--data=file.json"]));
    }

    #[test]
    fn value_equals_long_no_match() {
        let arg = Arg::new("--data").value("*.json");
        assert!(!arg.is_present(&["--data=file.txt"]));
    }

    // === Negative cases ===

    #[test]
    fn short_flag_not_present() {
        assert!(!Arg::new("-d").is_present(&["-f", "-g"]));
    }

    #[test]
    fn empty_args() {
        assert!(!Arg::new("-d").is_present(&[]));
    }

    #[test]
    fn glob_option_no_match() {
        assert!(!Arg::new("*mutation*").is_present(&["query { viewer }"]));
    }

    // === Substring glob on option ===

    #[test]
    fn glob_option_substring_match() {
        assert!(Arg::new("*mutation*").is_present(&["mutation { addComment }"]));
    }

    #[test]
    fn glob_option_substring_middle() {
        assert!(Arg::new("*mutation*").is_present(&["query='mutation { foo }'"]));
    }

    // === Long flag + value in two-arg form ===

    #[test]
    fn value_two_arg_long_flag() {
        let arg = Arg::new("--data").value("foo");
        assert!(arg.is_present(&["--data", "foo"]));
    }

    #[test]
    fn value_two_arg_long_flag_no_match() {
        let arg = Arg::new("--data").value("foo");
        assert!(!arg.is_present(&["--data", "bar"]));
    }

    // === ivalue with non-glob exact pattern ===

    #[test]
    fn ivalue_exact_case_insensitive() {
        let arg = Arg::new("-X").ivalue("POST");
        assert!(arg.is_present(&["-X", "post"]));
    }

    #[test]
    fn ivalue_exact_case_insensitive_match() {
        let arg = Arg::new("-X").ivalue("POST");
        assert!(arg.is_present(&["-X", "POST"]));
    }

    // === Value adjacency ===

    #[test]
    fn value_not_adjacent() {
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(!arg.is_present(&["-X", "--verbose", "POST"]));
    }

    // === Short flag bundled before value ===

    #[test]
    fn value_bundled_option_two_arg() {
        // -aX bundles -a and -X; POST is -X's value as next arg
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(arg.is_present(&["-aX", "POST"]));
    }

    // === Concatenated short: flag char not at position 1 ===

    #[test]
    fn value_concat_flag_not_first() {
        // -aXPOST: flag char X is at position 2, not position 1
        // Concatenated form only checks position 1, so this should not match
        let arg = Arg::new("-X").value("{POST,PUT,PATCH,DELETE}");
        assert!(!arg.is_present(&["-aXPOST"]));
    }

    // === exec with wrong value ===

    #[test]
    fn value_exec_wrong_value() {
        let arg = Arg::new("-exec").value("rm");
        assert!(!arg.is_present(&["-exec", "ls"]));
    }

    #[test]
    fn value_exec_no_value() {
        let arg = Arg::new("-exec").value("rm");
        assert!(!arg.is_present(&["-exec"]));
    }

    // === Long flag prefix without = ===

    #[test]
    fn long_flag_no_match_prefix() {
        assert!(!Arg::new("--data").is_present(&["--database"]));
    }

    // === Glob metacharacters: ? and [] ===

    #[test]
    fn glob_question_mark() {
        assert!(Arg::new("-?").is_present(&["-x"]));
    }

    #[test]
    fn glob_question_mark_no_match() {
        assert!(!Arg::new("-?").is_present(&["--xx"]));
    }

    #[test]
    fn glob_bracket_range() {
        assert!(Arg::new("-[abc]").is_present(&["-b"]));
    }

    #[test]
    fn glob_bracket_range_no_match() {
        assert!(!Arg::new("-[abc]").is_present(&["-d"]));
    }

    // === Cross-form: short option skips concat-long, long option skips concat-short ===

    #[test]
    fn short_option_no_concat_long_match() {
        let arg = Arg::new("-X").value("POST");
        assert!(!arg.is_present(&["-X=POST"]));
    }

    #[test]
    fn long_option_no_concat_short_match() {
        let arg = Arg::new("--method").value("POST");
        assert!(!arg.is_present(&["--methodPOST"]));
    }

    // === Glob: double-star ===

    #[test]
    fn glob_double_star() {
        assert!(Arg::new("**").is_present(&["anything"]));
    }

    #[test]
    fn glob_double_star_prefix() {
        assert!(Arg::new("**/foo").is_present(&["bar/foo"]));
    }

    #[test]
    fn glob_double_star_prefix_no_match() {
        assert!(!Arg::new("**/foo").is_present(&["bar/baz"]));
    }

    // === Glob: negated character class ===

    #[test]
    fn glob_negated_bracket() {
        assert!(Arg::new("-[!abc]").is_present(&["-d"]));
    }

    #[test]
    fn glob_negated_bracket_no_match() {
        assert!(!Arg::new("-[!abc]").is_present(&["-a"]));
    }

    // === Glob: escaped metacharacter via character class ===

    #[test]
    fn glob_escaped_star_bracket() {
        assert!(Arg::new("[*]").is_present(&["*"]));
    }

    #[test]
    fn glob_escaped_star_bracket_no_match() {
        assert!(!Arg::new("[*]").is_present(&["x"]));
    }

    // === Glob: backslash escape ===

    #[test]
    fn glob_backslash_escape_star() {
        assert!(Arg::new(r"\*").is_present(&["*"]));
    }

    #[test]
    fn glob_backslash_escape_star_no_match() {
        assert!(!Arg::new(r"\*").is_present(&["x"]));
    }
}
