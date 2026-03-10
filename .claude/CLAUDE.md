# claude-hooks

## Why

Single Rust binary replacing 12 bash Claude Code hooks. Reads JSON from stdin, evaluates shell commands against sequential security checks, and returns the first matching decision (allow/deny/ask) or passes through silently.

## What

- **Entry point**: `src/main.rs` — stdin JSON → evaluation → stdout JSON
- **Orchestration**: `src/evaluate.rs` — rule evaluation with Deny > Ask > Allow precedence
- **Shell parsing**: `src/command.rs` — brush-parser AST walking into `CompleteContext`
- **Rules**: `src/rules/*.rs` — `SimpleRule` (prefix-based) and `CompleteRule` (full AST analysis)
- **Schema**: `src/schema/*.rs` — `Outcome`, `HookInput`, `HookOutput`, rule types
- **Git utilities**: `src/utils/git.rs` — path classification (Trusted/Forked/Unknown), `-C` arg extraction

## How

### Build & Test

```sh
cargo build
cargo test
cargo insta test          # snapshot tests
cargo insta review        # review/accept snapshot changes
```

### Deploy

```sh
cargo install --path .    # build release to ~/.cargo/bin/
cp ~/.cargo/bin/claude-hooks ~/bin/claude-hooks  # deploy to active hooks path
```

The hook is configured in `~/.claude/settings.json` to run `~/bin/claude-hooks`.

### After Writing Code

```sh
cargo clippy --fix --allow-dirty --allow-staged && cargo fmt
cargo test
```

### Conventions

- Rules return `Some(Outcome)` to halt evaluation, `None` to continue
- Snapshot tests use `insta` with YAML format
- Test helpers: `check_t()` (trusted path), `check_bot()` (bot context)
- Error handling: `error-stack` + `thiserror`
- Extensive clippy lints configured in `Cargo.toml:19-75`

### Rust Guidelines

See `~/.config/le/claude/guidelines/rust.md` for detailed conventions covering:
- Code style, linting, error handling patterns
- `#[expect(...)]` over `#[allow(...)]`, always with `reason`
- Never `unwrap()` — use `expect()` with descriptive message
- Prefer `Target::from(value)` over `value.into()`
- Doc comments on all `pub`/`pub(crate)` items
- Tests co-located with code, AAA pattern, avoid low-value tests
- `mod.rs` files: only module declarations and re-exports
