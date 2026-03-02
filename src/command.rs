use regex::Regex;
use std::sync::LazyLock;

static SEGMENT_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&&|\|\||[;|]|\bdo\b|\bthen\b|\belse\b").expect("valid regex"));

static COMMAND_POSITION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|&&|\|\||[;|]|\bdo\b|\bthen\b|\belse\b)\s*").expect("valid regex")
});

/// Split a command string on `&&`, `||`, `;`, `|`, and shell keywords `do`, `then`, `else`.
pub fn split_segments(command: &str) -> Vec<&str> {
    SEGMENT_SPLIT.split(command).collect()
}

/// Check if `cmd` appears at a command position (start of line or after a separator),
/// not as an argument to echo/grep/cat etc.
pub fn has_command_at_position(command: &str, cmd: &str) -> bool {
    for m in COMMAND_POSITION.find_iter(command) {
        let rest = &command[m.end()..];
        if rest.starts_with(cmd)
            && rest[cmd.len()..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace)
        {
            return true;
        }
    }
    false
}

/// Iterate over command segments, yielding parsed git args for each segment
/// that contains a git command. Handles `-C <path>` transparently.
pub fn git_args_in_segments(command: &str) -> impl Iterator<Item = &str> {
    split_segments(command).into_iter().filter_map(|segment| {
        let (_, args) = parse_git(segment);
        if args.is_empty() { None } else { Some(args) }
    })
}

/// Parse a git command segment, extracting the `-C <path>` if present.
/// Returns `(path, remaining_args)` where path is empty string if no `-C`.
pub fn parse_git(segment: &str) -> (&str, &str) {
    let trimmed = segment.trim();
    let args = trimmed
        .strip_prefix("git")
        .filter(|rest| rest.is_empty() || rest.starts_with(' '))
        .map_or("", str::trim_start);
    if args.is_empty() {
        return ("", "");
    }
    let Some(after_c) = args.strip_prefix("-C ") else {
        return ("", args);
    };
    let after_c = after_c.trim_start();
    if let Some(content) = after_c.strip_prefix('"') {
        if let Some(end) = content.find('"') {
            let path = &content[..end];
            let rest = content[end + 1..].trim_start();
            return (path.trim_end_matches('/'), rest);
        }
    } else if let Some(content) = after_c.strip_prefix('\'')
        && let Some(end) = content.find('\'')
    {
        let path = &content[..end];
        let rest = content[end + 1..].trim_start();
        return (path.trim_end_matches('/'), rest);
    }
    match after_c.find(' ') {
        Some(pos) => {
            let path = &after_c[..pos];
            let rest = after_c[pos..].trim_start();
            (path.trim_end_matches('/'), rest)
        }
        None => (after_c.trim_end_matches('/'), ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic() {
        let segs = split_segments("ls && git status");
        assert_eq!(segs, vec!["ls ", " git status"]);
    }

    #[test]
    fn split_complex() {
        let segs = split_segments("git add file.txt && git commit -m 'test'");
        assert_eq!(segs, vec!["git add file.txt ", " git commit -m 'test'"]);
    }

    #[test]
    fn split_shell_keywords() {
        let segs = split_segments("for f in *.tmp; do rm $f; done");
        assert_eq!(segs.len(), 4);
    }

    #[test]
    fn command_position_basic() {
        assert!(has_command_at_position("rm file.txt", "rm"));
        assert!(has_command_at_position("ls && rm file.txt", "rm"));
        assert!(!has_command_at_position("echo rm is blocked", "rm"));
        assert!(!has_command_at_position("git rm file.txt", "rm"));
    }

    #[test]
    fn parse_git_no_path() {
        let (path, args) = parse_git("git status");
        assert_eq!(path, "");
        assert_eq!(args, "status");
    }

    #[test]
    fn parse_git_with_path() {
        let (path, args) = parse_git("git -C /var/mnt/e/Repos/Rust/caesura status");
        assert_eq!(path, "/var/mnt/e/Repos/Rust/caesura");
        assert_eq!(args, "status");
    }

    #[test]
    fn parse_git_quoted_path() {
        let (path, args) = parse_git("git -C \"/var/mnt/e/Repos/Rust/caesura\" status");
        assert_eq!(path, "/var/mnt/e/Repos/Rust/caesura");
        assert_eq!(args, "status");
    }

    #[test]
    fn parse_git_single_quoted_path() {
        let (path, args) = parse_git("git -C '/var/mnt/e/Repos/Rust/caesura' status");
        assert_eq!(path, "/var/mnt/e/Repos/Rust/caesura");
        assert_eq!(args, "status");
    }

    #[test]
    fn parse_git_trailing_slash() {
        let (path, args) = parse_git("git -C /var/mnt/e/Repos/Rust/caesura/ status");
        assert_eq!(path, "/var/mnt/e/Repos/Rust/caesura");
        assert_eq!(args, "status");
    }

    #[test]
    fn parse_non_git() {
        let (path, args) = parse_git("ls -la");
        assert_eq!(path, "");
        assert_eq!(args, "");
    }
}
