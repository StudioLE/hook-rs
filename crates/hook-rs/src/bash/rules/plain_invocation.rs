//! Shared check for bare `--help` and `--version` invocations.

use crate::prelude::*;

/// Commands whose operands are code to run, a package to fetch, or a remote host.
///
/// - `--help` after such an operand reaches that code, not the command
const EXCLUDED_COMMANDS: &[&str] = &[
    // Shells and interpreters
    "awk",
    "bash",
    "bun",
    "dash",
    "deno",
    "fish",
    "gawk",
    "java",
    "julia",
    "ksh",
    "lua",
    "mawk",
    "nawk",
    "node",
    "perl",
    "php",
    "pwsh",
    "python",
    "python3",
    "Rscript",
    "ruby",
    "sh",
    "zsh",
    // Shell builtins
    "builtin",
    "command",
    "eval",
    "exec",
    "source",
    // Package and script runners
    "bunx",
    "cargo",
    "dotnet",
    "flatpak",
    "go",
    "gradle",
    "hatch",
    "mise",
    "mvn",
    "nix",
    "npm",
    "npx",
    "pdm",
    "pipx",
    "pnpm",
    "pnpx",
    "poetry",
    "rustup",
    "uv",
    "uvx",
    "yarn",
    // Containers
    "distrobox",
    "docker",
    "podman",
    "toolbox",
    // Wrappers
    "bwrap",
    "busybox",
    "chroot",
    "doas",
    "env",
    "fakeroot",
    "firejail",
    "flock",
    "gdb",
    "ionice",
    "ltrace",
    "nice",
    "nohup",
    "nsenter",
    "perf",
    "pkexec",
    "run0",
    "runuser",
    "script",
    "setsid",
    "stdbuf",
    "strace",
    "su",
    "sudo",
    "systemd-run",
    "taskset",
    "time",
    "timeout",
    "unshare",
    "valgrind",
    "watch",
    "xargs",
    // Remote
    "mosh",
    "nc",
    "ssh",
    "telnet",
];

/// Is the command a plain name followed only by plain `components`?
///
/// - Name and `components` are non-empty ASCII `[A-Za-z0-9_-]` not starting with `-`
/// - Rejects paths, quoted words, flags, variables, and substitutions, since args are raw words
/// - Rejects [`EXCLUDED_COMMANDS`]
/// - Rejects env var prefixes and heredocs
pub fn is_plain_invocation(ctx: &BashRuleContext, components: &[String]) -> bool {
    let simple = ctx.simple;
    simple.env_vars.is_empty()
        && !simple.has_heredoc
        && is_plain_component(&simple.name)
        && !EXCLUDED_COMMANDS.contains(&simple.name.as_str())
        && components.iter().all(|c| is_plain_component(c))
}

/// Is `word` a non-empty ASCII `[A-Za-z0-9_-]` word not starting with `-`?
fn is_plain_component(word: &str) -> bool {
    !word.is_empty()
        && !word.starts_with('-')
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_plain_component_alphanumeric() {
        assert!(is_plain_component("aws"));
        assert!(is_plain_component("S3api"));
        assert!(is_plain_component("sub-cmd"));
        assert!(is_plain_component("my_tool"));
        assert!(is_plain_component("_private"));
    }

    #[test]
    fn is_plain_component_flag() {
        assert!(!is_plain_component("-x"));
        assert!(!is_plain_component("--help"));
    }

    #[test]
    fn is_plain_component_empty() {
        assert!(!is_plain_component(""));
    }

    #[test]
    fn is_plain_component_path() {
        assert!(!is_plain_component("./deploy"));
        assert!(!is_plain_component("foo.sh"));
        assert!(!is_plain_component("s3://bucket"));
        assert!(!is_plain_component(r"a\b"));
    }

    #[test]
    fn is_plain_component_variable() {
        assert!(!is_plain_component("$x"));
        assert!(!is_plain_component("${x}"));
        assert!(!is_plain_component("$(id)"));
        assert!(!is_plain_component("`id`"));
    }

    #[test]
    fn is_plain_component_quoted() {
        assert!(!is_plain_component("'foo'"));
        assert!(!is_plain_component("\"foo\""));
    }

    /// Unicode letters can look like ASCII ones.
    #[test]
    fn is_plain_component_unicode() {
        assert!(!is_plain_component("bаsh"));
        assert!(!is_plain_component("café"));
    }
}
