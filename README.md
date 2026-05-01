# hook-rs

A tool to check and enforce permissions for Claude Code.

- Bash syntax aware command analysis
- Glob based git path trust classification
- Tilde aware Read, Grep, and Glob path auto-allowing

## How it Works

- Implemented as a [PreToolUse hook](https://docs.anthropic.com/en/docs/claude-code/hooks) for Claude Code
- Reads JSON from stdin
- Evaluates all matching rules
- Returns the highest-priority decision: Deny > Ask > Allow
- Or, passes through to the default permission system.

## Read, Grep, and Glob Rules

Allowed and excluded paths are defined in `settings.yaml` using `.gitignore` style glob patterns.

Refer to the [glob syntax guide](#glob-syntax).

## Bash Rules

<details>
<summary>Allow Read-Only Commands</summary>

Allow read-only commands:
- `base64`
- `basename`
- `cat`
- `column`
- `command`
- `cut`
- `dirname`
- `echo`
- `file`
- `fmt`
- `head`
- `jq`
- `less`
- `ls`
- `readlink`
- `realpath`
- `rg`
- `stat`
- `tail`
- `tr`
- `tree`
- `type`
- `uniq`
- `wc`
- `which`
- `xxd`

Allow without in-place flags (`-i`, `--in-place`):
- `sort` (without `-o`, `--output`)
- `yq`

[source: `allow_safe.rs`](src/bash/rules/allow_safe.rs)

Allow `fd` without exec flags (`-x`, `--exec`, `-X`, `--exec-batch`).

[source: `fd.rs`](src/bash/rules/fd.rs)

Allow read-only git subcommands:
- `git check-ignore`
- `git describe`
- `git diff`
- `git fetch`
- `git log`
- `git ls-tree`
- `git merge-base`
- `git mv`
- `git rev-parse`
- `git rm`
- `git show`
- `git status`

Allow bare or with read-only flags only (e.g. `-a`, `--list`, `-v`, `--contains`, `--merged`):
- `git branch`
- `git tag`
- `git remote`

[source: `git_allow.rs`](src/bash/rules/git_allow.rs)

</details>

<details>
<summary>Deny Unnecessary Destructive Commands</summary>

Deny all forms of `rm`. Suggests `git rm -f` or `git clean -f <file>` instead.

[source: `rm.rs`](src/bash/rules/rm.rs)

Deny `find -delete` and `find -exec rm` / `find -execdir rm`.

[source: `find.rs`](src/bash/rules/find.rs)

Deny `fd -x rm` / `fd --exec rm` / `fd -X rm` / `fd --exec-batch rm`.

[source: `fd.rs`](src/bash/rules/fd.rs)

</details>

<details>
<summary>Deny Destructive Git Operations</summary>

Deny destructive git operations:
- `git reset --hard`
- `git stash pop`
- `git stash drop`
- `git stash clear`
- `git clean -d` (any flag combo containing `-d`)
- `git checkout --` (discarding changes)

[source: `git_deny.rs`](src/bash/rules/git_deny.rs)

</details>

<details>
<summary>Trusted Git Paths</summary>

Handle `git -C <path>` by combining path trust classification with subcommand analysis.

- Destructive subcommands are denied regardless of path trust.
- Safe subcommands are allowed only in trusted paths.

[source: `git_c.rs`](src/bash/rules/git_c.rs)

- Deny `cd <path>` chained with `git` via any operator (`&&`, `||`, `;`).
- Suggests using `git -C <path>` instead.

[source: `cd_git.rs`](src/bash/rules/cd_git.rs)

</details>

<details>
<summary>Chained push</summary>

Deny `git push` when part of a compound command. Requires it to be run standalone.

[source: `chained_push.rs`](src/bash/rules/chained_push.rs)

</details>

<details>
<summary>GitHub CLI</summary>

Allow read-only `gh` commands:
- `gh run list`
- `gh run view`
- `gh release list`
- `gh api` (without data flags or write methods)

Ask for approval on write operations:
- `gh pr comment`
- `gh api` with data flags (`-d`, `-f`, `-F`, `--input`)
- `gh api` with write methods (`-X POST/PUT/PATCH/DELETE`)
- `gh api graphql` with mutations

[source: `gh.rs`](src/bash/rules/gh.rs)

</details>

<details>
<summary>Python</summary>

Deny inline Python (`-c` or heredoc) exceeding 1000 characters or 20 lines.

[source: `long_python.rs`](src/bash/rules/long_python.rs)

</details>

<details>
<summary>Insta review</summary>

Deny `cargo insta review` with heredoc input to prevent faking interactive input.

[source: `insta.rs`](src/bash/rules/insta.rs)

</details>

## Parsers

Bash rules classify command arguments using three approaches in increasing order of strictness. All three strip shell quotes and handle short-flag bundling (e.g. `-an5`).

### `ArgMatcher` - Pattern matching on individual arguments

Lightweight boolean matcher embedded directly in `BashRule` fields (`with_any`, `with_all`, `without_any`). Checks whether a flag or value is present in the argument list using glob patterns. No schema required.

Recognizes three flag-value forms: two-arg (`-X POST`), concatenated short (`-XPOST`), and equals long (`--data=foo`).

Best for simple presence/absence checks where you don't need structured parsing.

```rust
BashRule {
    with_any: Some(vec![
        ArgMatcher::new("-X").ivalue("{POST,PUT,PATCH,DELETE}"),
        ArgMatcher::new("--data"),
    ]),
    without_any: Some(vec![ArgMatcher::new("graphql")]),
    ..
}
```

[source: `arg_matcher.rs`](src/bash/arg_matcher.rs)

### `ArgParser` - Flat schema-aware classification

Walks a list of arguments against an `ArgSchema` that declares which flags are boolean and which take a value. Rejects unknown flags. Returns `Vec<Arg>` where each element is `Flag`, `FlagPair`, `Operand`, or `Separator`.

Best for rules that receive pre-sliced arguments and need to distinguish flags from positional values without subcommand awareness.

```rust
let settings = ArgParserSettings {
    schema: ArgSchema {
        bool_flags: vec![],
        value_flags: vec![String::from("-b")],
    },
    unquote: true,
};
let parsed: Vec<Arg> = ArgParser::new(settings).parse(args)?;
```

[source: `utils/parsers/arg/`](src/utils/parsers/arg/)

### `CommandParser` - Hierarchical command parsing

Walks the full token stream against a recursive `CommandSchema` tree. Each node defines its own options, operand constraints, and subcommands. Supports value validation via glob, regex, and exact-set constraints. Returns `Vec<ParsedCommand>` where index 0 is the root command, index 1 is the first subcommand, etc.

Best for rules that need the full command hierarchy, value validation, or operand count enforcement.

```rust
let schema = CommandSchemaBuilder::new("git")
    .with_subcommand(
        CommandSchemaBuilder::new("worktree")
            .with_subcommand(
                CommandSchemaBuilder::new("add")
                    .with_option(
                        OptionSchemaBuilder::new(["-b"])
                            .with_value(ValueConstraint::Any)
                            .build(),
                    )
                    .build(),
            )
            .build(),
    )
    .build();
let parsed: Vec<ParsedCommand> = CommandParser::new(schema).parse(args)?;
```

[source: `utils/parsers/command/`](src/utils/parsers/command/)

## Install

### Homebrew

```bash
brew install StudioLE/tap/hook-rs
```

### From Source

```bash
cargo install --path .
```

## Configuration

### Setup Hooks

Enable the `PreToolUse` hooks in `~/.claude/settings.json`

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "hook-rs bash" }]
      },
      {
        "matcher": "Glob",
        "hooks": [{ "type": "command", "command": "hook-rs glob" }]
      },
      {
        "matcher": "Grep",
        "hooks": [{ "type": "command", "command": "hook-rs grep" }]
      },
      {
        "matcher": "Read",
        "hooks": [{ "type": "command", "command": "hook-rs read" }]
      }
    ]
  }
}
```

### Settings

Refer to the [glob syntax guide](#glob-syntax).

Settings are optional. If missing, defaults to empty.

Create a `~/.config/hook-rs/settings.yaml` file with your settings:

```yaml
git:
  paths:
  # Trust all repos in ~/repos
  - ~/repos/**
  # Exclude all repos in ~/repos/forked
  #
  - !~/repos/forked/**
  # Trust all repos in ~/repos/forked/my-fork
  - ~/repos/forked/my-fork/**

read:
  paths:
  # Allow reading the cargo registry
  - ~/.cargo/registry/src/**
  # Allow reading the rustup toolchain
  - ~/.rustup/toolchains/**
  # Allow reading any file in /path/to/repos
  - /path/to/repos/**
  # Allow reading any README.md
  - README.md
  # Exclude .env
  - !.env
  - !.env.*
```

> [!NOTE]
>
> While technically this is YAML tag syntax:
>
> ```yaml
>   - !.env
> ```
>
> A pre-processor automatically converts it to a YAML string:
>
> ```yaml
>   - "!.env"
> ```

### Glob Syntax

- Last match wins
- `!` prefix excludes
- `*` matches zero or more characters except `/`
- `?` matches any single character except `/`
- `**` recursively matches directories
- `{a,b}` matches `a` or `b` where `a` and `b` are arbitrary glob patterns (Nesting `{...}` is not currently allowed)
- `[ab]` matches `a` or `b` where `a` and `b` are characters. Use [!ab] to match any character except for a and b.
- Metacharacters such as `*` and `?` can be escaped with character class notation. e.g., `[*]` matches `*`.

> [!NOTE]
>
> `**` recursively matches directories but are only legal in three situations:
>
> 1. If the glob starts with `**/`, then it matches all directories.
>
> For example, `**/foo` matches `foo` and `bar/foo` but not `foo/bar`.
>
> 2. If the glob ends with `/**`, then it matches all sub-entries.
>
> For example, `foo/**` matches `foo/a` and `foo/a/b`, but not `foo`.
>
> 3. If the glob contains `/**/` anywhere within the pattern, then it matches zero or more directories.
>
> Using `**` anywhere else is illegal
>
> The glob `**` is allowed and means "match everything".
