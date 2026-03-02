# claude-hooks

A single Rust binary that replaces 12 bash [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks). Reads JSON from stdin, runs checks in order, outputs the first matching decision. No output = passthrough.

## Checks

| # | Module | Decision | What it does |
|---|--------|----------|--------------|
| 1 | `gh_cli` | allow / ask | Auto-allow read-only gh commands; ask for writes, mutations, PR comments |
| 2 | `rm` | deny | Block `rm` and `git clean -d` (allow `/tmp/` without traversal) |
| 3 | `git_approval` | allow / ask | Safe subcommands + `-C` path classification + branch/tag/remote flag validation |
| 4 | `cd_git` | deny | Block `cd && git` compounds |
| 5 | `git_stash` | deny | Block `stash pop`, `drop`, `clear` |
| 6 | `git_reset` | deny | Block `reset --hard` |
| 7 | `git_checkout` | deny | Block `checkout HEAD --` and `checkout --` (discard changes) |
| 8 | `find_delete` | deny | Block `find -delete` and `find -exec rm` |
| 9 | `chained_push` | deny | Block non-standalone `git push` |
| 10 | `echo_separator` | deny | Block chained echo separators (`&& echo "---"`) |
| 11 | `insta_review` | deny | Block `cargo insta review` with heredoc input |
| 12 | `long_python` | deny | Block inline Python >1000 chars or >20 lines |

## Install

```sh
cargo install --path .
```

This installs `claude-hooks` to `~/.cargo/bin/`.

## Configuration

In `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.cargo/bin/claude-hooks"
          }
        ]
      }
    ]
  }
}
```

## Testing

```sh
cargo test
```

Tests use [insta](https://insta.rs) yaml snapshots. After adding new test cases:

```sh
cargo insta test
cargo insta review
```

## Manual testing

```sh
echo '{"tool_input":{"command":"git status"}}' | claude-hooks
# → {"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow",...}}

echo '{"tool_input":{"command":"rm -rf /"}}' | claude-hooks
# → {"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny",...}}

echo '{"tool_input":{"command":"ls -la"}}' | claude-hooks
# → (no output = passthrough)
```
