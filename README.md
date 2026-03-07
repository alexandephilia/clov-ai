# clov: Token Omitter for LLM Workflows

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.25.0-blue.svg)](https://github.com/alexandephilia/clov-ai/releases/tag/v0.25.0)

**A high-performance CLI proxy that cuts token waste before it hits your LLM context.**

![clov preview](img_1.jpg)

clov sits between your shell and your model. It intercepts command output, filters the noise, and hands back only what matters. Token reduction varies by command and project, but the difference is consistent enough to notice from day one.

## Token Savings (30-min Claude Code Session)

![clov preview 2](img_2.jpg)

Typical session without clov: **~150,000 tokens**
With clov: **~45,000 tokens** -> **70% reduction**

| Operation                 | Frequency | Standard     | clov        | Savings  |
| ------------------------- | --------- | ------------ | ----------- | -------- |
| `ls` / `tree`             | 10x       | 2,000        | 400         | -80%     |
| `cat` / `read`            | 20x       | 40,000       | 12,000      | -70%     |
| `grep` / `rg`             | 8x        | 16,000       | 3,200       | -80%     |
| `git status`              | 10x       | 3,000        | 600         | -80%     |
| `git diff`                | 5x        | 10,000       | 2,500       | -75%     |
| `git log`                 | 5x        | 2,500        | 500         | -80%     |
| `git add/commit/push`     | 8x        | 1,600        | 120         | -92%     |
| `npm test` / `cargo test` | 5x        | 25,000       | 2,500       | -90%     |
| `ruff check`              | 3x        | 3,000        | 600         | -80%     |
| `pytest`                  | 4x        | 8,000        | 800         | -90%     |
| `go test`                 | 3x        | 6,000        | 600         | -90%     |
| `docker ps`               | 3x        | 900          | 180         | -80%     |
| **Total**                 |           | **~118,000** | **~23,900** | **-80%** |

> Numbers are estimates based on medium-sized TypeScript/Rust projects. Results vary.

## Installation

### Pre-Installation Check

Check if clov is already on your system before doing anything:

```bash
clov --version        # Check if installed
clov gain             # Confirm it is the Token Omitter
which clov            # Check installation path
```

If `clov gain` works, skip ahead to [Quick Start](#quick-start).

### Homebrew (macOS/Linux)

```bash
brew tap alexandephilia/clov
brew install clov
```

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/alexandephilia/clov-ai/refs/heads/main/install.sh | sh
```

> clov installs to `~/.local/bin` by default. If that is not in your PATH:
>
> ```bash
> echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
> ```

After installing, verify it works:

```bash
clov gain  # Must show token savings stats, not "command not found"
```

### Manual Installation

```bash
cargo install --git https://github.com/alexandephilia/clov-ai
```

### Pre-built Binaries

Download from [clov-ai/releases](https://github.com/alexandephilia/clov-ai/releases):

- macOS: `clov-x86_64-apple-darwin.tar.gz` / `clov-aarch64-apple-darwin.tar.gz`
- Linux: `clov-x86_64-unknown-linux-gnu.tar.gz` / `clov-aarch64-unknown-linux-gnu.tar.gz`
- Windows: `clov-x86_64-pc-windows-msvc.zip`

## Quick Start

```bash
# 1. Verify the installation
clov gain  # Must show token stats, not "command not found"

# 2. Initialize for Claude Code (hook-first mode is recommended)
clov init --global
# -> Installs the rewrite hook + creates a slim CLOV.md (10 lines, 99.5% token savings)
# -> Follow the printed instructions to register the hook in ~/.claude/settings.json

# 3. Confirm it is working
clov git status   # Should produce compact output
clov init --show  # Verify the hook is installed and executable

# Alternative modes:
# clov init --global --claude-md  # Legacy: full injection (137 lines)
# clov init                       # Local project only (./CLAUDE.md)
```

**v0.25.0**: Hook-first install removes ~2,000 tokens from Claude's context while keeping full clov functionality through command rewriting.

## Global Flags

```bash
-u, --ultra-compact    # ASCII icons, inline format (extra token savings)
-v, --verbose          # Increase verbosity (-v, -vv, -vvv)
```

## Commands

### Files

```bash
clov ls .                        # Token-optimized directory tree
clov read file.rs                # Smart file reading
clov read file.rs -l aggressive  # Signatures only, strips bodies
clov smart file.rs               # 2-line heuristic code summary
clov find "*.rs" .               # Compact find results
clov grep "pattern" .            # Grouped search results
```

### Git

```bash
clov git status                  # Compact status
clov git log -n 10               # One-line commits
clov git diff                    # Condensed diff
clov git add                     # -> "ok ✓"
clov git commit -m "msg"         # -> "ok ✓ abc1234"
clov git push                    # -> "ok ✓ main"
clov git pull                    # -> "ok ✓ 3 files +10 -2"
```

### Commands

```bash
clov test cargo test             # Show failures only (-90% tokens)
clov err npm run build           # Errors and warnings only
clov summary <long command>      # Heuristic summary
clov log app.log                 # Deduplicated logs
clov gh pr list                  # Compact PR listing
clov gh pr view 42               # PR details + checks summary
clov gh issue list               # Compact issue listing
clov gh run list                 # Workflow run status
clov wget https://example.com   # Download, strip progress bars
clov config                      # Show config (--create to generate)
clov ruff check                  # Python linting (JSON, 80% reduction)
clov pytest                      # Python tests (failures only, 90% reduction)
clov pip list                    # Python packages (auto-detect uv, 70% reduction)
clov go test                     # Go tests (NDJSON, 90% reduction)
clov golangci-lint run           # Go linting (JSON, 85% reduction)
```

### Data and Analytics

```bash
clov json config.json            # Structure without values
clov deps                        # Dependencies summary
clov env -f AWS                  # Filtered env vars

# Token savings analytics
clov gain                        # Summary stats with total exec time
clov gain --graph                # ASCII graph of last 30 days
clov gain --history              # Recent command history (10)
clov gain --quota --tier 20x     # Monthly quota analysis (pro/5x/20x)

# Temporal breakdowns
clov gain --daily                # Day-by-day with avg execution time
clov gain --weekly               # Week-by-week breakdown
clov gain --monthly              # Month-by-month breakdown
clov gain --all                  # All breakdowns combined

# Export
clov gain --all --format json    # JSON for APIs/dashboards
clov gain --all --format csv     # CSV for Excel/analysis
```


### Discover: Find Missed Savings

Scans your Claude Code session history and shows where clov would have saved tokens. Useful for:

- Seeing exactly how many tokens you left on the table
- Finding which commands you keep running without clov
- Spotting unhandled commands worth turning into clov features

```bash
clov discover                    # Current project, last 30 days
clov discover --all              # All Claude Code projects
clov discover --all --since 7    # Last 7 days across all projects
clov discover -p aristote        # Filter by project name (substring)
clov discover --format json      # Machine-readable output
```

Example output:

```
╔══════════════════════════════════════════════════════════╗
║        clov discover - Savings Opportunities             ║
╠══════════════════════════════════════════════════════════╣
║  Scanned : 142 sessions · last 30 days                  ║
║  Commands: 1,786 Bash invocations                       ║
║  Via clov: 108  (6%)                                    ║
╠══════════════════════════════════════════════════════════╣
║  MISSED SAVINGS - commands clov already handles          ║
╠══════════════════════════════════════════════════════════╣
║  Command        Count   clov Equivalent    Est. Savings  ║
║  ────────────────────────────────────────────────────── ║
║  git log          434   clov git           ~55.9K tokens ║
║  cargo test       203   clov cargo         ~49.9K tokens ║
║  ls -la           107   clov ls            ~11.8K tokens ║
║  gh pr             80   clov gh            ~10.4K tokens ║
║  ────────────────────────────────────────────────────── ║
║  Total: 986 commands  ->  ~143.9K tokens recoverable     ║
╠══════════════════════════════════════════════════════════╣
║  TOP UNHANDLED - worth opening an issue?                 ║
╠══════════════════════════════════════════════════════════╣
║  Command        Count   Example                          ║
║  ────────────────────────────────────────────────────── ║
║  git checkout      84   git checkout feature/my-branch  ║
║  cargo run         32   cargo run -- gain --help         ║
║  ────────────────────────────────────────────────────── ║
║  -> github.com/alexandephilia/clov-ai/issues            ║
╚══════════════════════════════════════════════════════════╝
```

### Containers

```bash
clov docker ps                   # Compact container list
clov docker images               # Compact image list
clov docker logs <container>     # Deduplicated logs
clov kubectl pods                # Compact pod list
clov kubectl logs <pod>          # Deduplicated logs
clov kubectl services            # Compact service list
```

### JavaScript and TypeScript Stack

```bash
clov lint                         # ESLint grouped by rule/file
clov lint biome                   # Works with other linters too
clov tsc                          # TypeScript errors grouped by file
clov next build                   # Next.js build compact output
clov prettier --check .           # Files that need formatting
clov vitest run                   # Test failures only
clov playwright test              # E2E results, failures only
clov prisma generate              # Schema generation without ASCII art
clov prisma migrate dev --name x  # Migration summary
clov prisma db-push               # Schema push summary
```

### Python and Go Stack

```bash
# Python
clov ruff check                   # Ruff linter (JSON, 80% reduction)
clov ruff format                  # Ruff formatter (text filter)
clov pytest                       # Test failures with state machine parser (90% reduction)
clov pip list                     # Package list (auto-detect uv, 70% reduction)
clov pip install <package>        # Install with compact output
clov pip outdated                 # Outdated packages (85% reduction)

# Go
clov go test                      # NDJSON streaming parser (90% reduction)
clov go build                     # Build errors only (80% reduction)
clov go vet                       # Vet issues (75% reduction)
clov golangci-lint run            # JSON grouped by rule (85% reduction)
```

## Examples

### Before and After

**Directory listing:**

```
# ls -la (45 lines, ~800 tokens)
drwxr-xr-x  15 user  staff    480 Jan 23 10:00 .
drwxr-xr-x   5 user  staff    160 Jan 23 09:00 ..
-rw-r--r--   1 user  staff   1234 Jan 23 10:00 Cargo.toml
...

# clov ls (12 lines, ~150 tokens)
my-project/
├── src/ (8 files)
│   ├── main.rs
│   └── lib.rs
├── Cargo.toml
└── README.md
```

**Git operations:**

```
# git push (15 lines, ~200 tokens)
Enumerating objects: 5, done.
Counting objects: 100% (5/5), done.
Delta compression using up to 8 threads
...

# clov git push (1 line, ~10 tokens)
ok ✓ main
```

**Test output:**

```
# cargo test (200+ lines on failure)
running 15 tests
test utils::test_parse ... ok
test utils::test_format ... ok
...

# clov test cargo test (only failures, ~20 lines)
FAILED: 2/15 tests
  ✗ test_edge_case: assertion failed at src/lib.rs:42
  ✗ test_overflow: panic at src/utils.rs:18
```

## How It Works

```
  ╔══════════════════════════════════════════════════════════════════════╗
  ║  WITHOUT clov                                                        ║
  ╠══════════════════════════════════════════════════════════════════════╣
  ║                                                                      ║
  ║  ┌────────────┐  git status  ┌──────────┐  git status  ┌─────────┐  ║
  ║  │ Claude LLM │ ───────────► │  Shell   │ ───────────► │   git   │  ║
  ║  └────────────┘              └──────────┘              └─────────┘  ║
  ║        ▲                                                     │       ║
  ║        │           ~2,000 tokens  (raw, unfiltered)          │       ║
  ║        └─────────────────────────────────────────────────────┘       ║
  ║                                                                      ║
  ╠══════════════════════════════════════════════════════════════════════╣
  ║  WITH clov                                                           ║
  ╠══════════════════════════════════════════════════════════════════════╣
  ║                                                                      ║
  ║  ┌────────────┐  git status  ┌──────────┐  git status  ┌─────────┐  ║
  ║  │ Claude LLM │ ───────────► │   clov   │ ───────────► │   git   │  ║
  ║  └────────────┘              │  (proxy) │              └─────────┘  ║
  ║        ▲                     └──────────┘                   │       ║
  ║        │                          │   ~2,000 tokens raw ◄───┘       ║
  ║        │                          ▼                                  ║
  ║        │              ┌───────────────────────────────┐              ║
  ║        │              │ filter · group · dedup · trim │              ║
  ║        │              └───────────────────────────────┘              ║
  ║        │    ~200 tokens  (distilled, signal-only)                    ║
  ║        └─────────────────────────────────────────────────────────────║
  ║                                                                      ║
  ╚══════════════════════════════════════════════════════════════════════╝
```

Four reduction strategies applied per command type:

1. **Smart Filtering**: strips noise, comments, blank lines, ANSI codes, boilerplate
2. **Grouping**: aggregates related items, errors by type, files by directory
3. **Truncation**: keeps the relevant context, cuts the repetition
4. **Deduplication**: collapses repeated log lines into single entries with counts

## Configuration

### Installation Modes

| Command                    | Scope  | Hook | CLOV.md       | CLAUDE.md        | Tokens in Context | Use Case                                 |
| -------------------------- | ------ | ---- | ------------- | ---------------- | ----------------- | ---------------------------------------- |
| `clov init -g`             | Global | yes  | yes (10 lines) | @CLOV.md        | ~10               | Recommended: all projects, automatic     |
| `clov init -g --claude-md` | Global | no   | no            | Full (137 lines) | ~2000             | Legacy compatibility                     |
| `clov init -g --hook-only` | Global | yes  | no            | Nothing          | 0                 | Minimal setup, hook only                 |
| `clov init`                | Local  | no   | no            | Full (137 lines) | ~2000             | Single project, no hook                  |

```bash
clov init --show         # Show current configuration
clov init -g             # Install hook + CLOV.md (recommended)
clov init -g --claude-md # Legacy: full injection into CLAUDE.md
clov init                # Local project: full injection into ./CLAUDE.md
```

### Installation Flags

**settings.json control:**

```bash
clov init -g                 # Default: prompt to patch [y/N]
clov init -g --auto-patch    # Patch settings.json without prompting
clov init -g --no-patch      # Skip patching, show manual instructions
```

**Mode control:**

```bash
clov init -g --claude-md     # Legacy: full 137-line injection (no hook)
clov init -g --hook-only     # Hook only, no CLOV.md
```

**Uninstall:**

```bash
clov init -g --uninstall     # Remove all clov artifacts
```

**What is settings.json?**
Claude Code's configuration file. It registers the clov hook, which rewrites commands like `git status` to `clov git status` before execution. Without this registration, Claude ignores the hook.

**Backup behavior:**
clov creates `~/.claude/settings.json.bak` before making changes. To revert:

```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

**Migration**: If you used `clov init -g` with the old 137-line injection, just re-run `clov init -g` to migrate to the hook-first approach.

**Example: 3-day session (`clov gain --all`):**

```
╔══════════════════════════════════════════════════════╗
║           clov gain - Token Savings                  ║
╠══════════════════════════════════════════════════════╣
║  Total commands  :   133                             ║
║  Input tokens    :  30.5K                            ║
║  Output tokens   :  10.7K                            ║
║  Tokens saved    :  25.3K  (83.0%)                   ║
╠══════════════════════════════════════════════════════╣
║  By Command                                          ║
║  ────────────────────────────────────────────────── ║
║  Command               Count    Saved     Avg%       ║
║  clov git status          41    17.4K    82.9%       ║
║  clov git push            54     3.4K    91.6%       ║
║  clov grep                15     3.2K    26.5%       ║
║  clov ls                  23     1.4K    37.2%       ║
╠══════════════════════════════════════════════════════╣
║  Daily Savings (last 30 days)                        ║
║  ────────────────────────────────────────────────── ║
║  01-23 │███████████████████              6.4K        ║
║  01-24 │██████████████████               5.9K        ║
║  01-25 │                                   18        ║
║  01-26 │████████████████████████████████ 13.0K       ║
╚══════════════════════════════════════════════════════╝
```

### Custom Database Path

By default, clov stores tracking data in `~/.local/share/clov/history.db`. To override:

**Environment variable (highest priority):**

```bash
export CLOV_DB_PATH="/path/to/custom.db"
```

**Config file (`~/.config/clov/config.toml`):**

```toml
[tracking]
database_path = "/path/to/custom.db"
```

Priority: `CLOV_DB_PATH` env var > `config.toml` > default location.

### Tee: Full Output Recovery

When clov filters a command, the LLM loses the raw failure details and may re-run the same command multiple times. The tee feature saves the unfiltered output to a file so the agent can read it directly instead of re-executing.

On failure, clov writes full output to `~/.local/share/clov/tee/` and prints a one-line hint:

```
✓ cargo test: 15 passed (1 suite, 0.01s)
[full output: ~/.local/share/clov/tee/1707753600_cargo_test.log]
```

The agent reads the file. No re-run needed.

**Default behavior**: tee only on failure (exit code != 0), skip outputs under 500 chars.

**Config (`~/.config/clov/config.toml`):**

```toml
[tee]
enabled = true          # default: true
mode = "failures"       # "failures" (default), "always", or "never"
max_files = 20          # max files to keep, oldest rotated out
max_file_size = 1048576 # 1MB per file max
# directory = "/custom/path"  # override default location
```

**Environment overrides:**

- `CLOV_TEE=0`: disable tee entirely
- `CLOV_TEE_DIR=/path`: override output directory

**Supported commands**: cargo (build/test/clippy/check/install/nextest), vitest, pytest, lint (eslint/biome/ruff/pylint/mypy), tsc, go (test/build/vet), err, test.

## Auto-Rewrite Hook (Recommended)

The best way to use clov is the auto-rewrite hook for Claude Code. CLAUDE.md instructions get ignored by subagents. The hook does not. It intercepts Bash commands and rewrites them to their clov equivalents before the shell sees them.

**Result**: 100% clov adoption across all conversations and subagents, zero token overhead in Claude's context.

### What Are Hooks?

Claude Code hooks are scripts that run before or after Claude executes a command. clov registers a **PreToolUse** hook that intercepts Bash commands and rewrites them silently, e.g. `git status` becomes `clov git status` before execution. Claude only receives the filtered output.

**Why settings.json?** Claude Code reads `~/.claude/settings.json` to find registered hooks. Without that entry, it does not know the hook exists.

**Is it safe?** Yes. clov creates a backup before any change. The hook only modifies command strings. It does not delete files or access secrets. You can read the hook at `~/.claude/hooks/clov-rewrite.sh` any time.

### How It Works

The hook runs as a Claude Code [PreToolUse hook](https://docs.anthropic.com/en/docs/claude-code/hooks). When Claude is about to run `git status`, the hook rewrites it to `clov git status` before it reaches the shell.

```
  ┌──────────────────────────────────────────────────────┐
  │  Claude Code issues:  git status                     │
  └───────────────────────────┬──────────────────────────┘
                              │
              ┌───────────────▼───────────────────┐
              │      ~/.claude/settings.json       │
              │    PreToolUse hook registered      │
              └───────────────┬───────────────────┘
                              │
              ┌───────────────▼───────────────────┐
              │         clov-rewrite.sh            │
              │                                    │
              │   "git status"                     │
              │        ──────────────────────►     │
              │   "clov git status"                │  <- silent rewrite
              └───────────────┬───────────────────┘
                              │
              ┌───────────────▼───────────────────┐
              │         clov  (Rust binary)        │
              │  · runs the real git status        │
              │  · filters and compresses output   │
              └───────────────┬───────────────────┘
                              │
  ┌───────────────────────────▼──────────────────────────┐
  │  Claude receives:  "3 modified, 1 untracked ✓"       │
  │                    not 50 lines of raw git output     │
  └──────────────────────────────────────────────────────┘
```

### Quick Install (Automated)

```bash
clov init -g
# -> Installs hook to ~/.claude/hooks/clov-rewrite.sh (with executable permissions)
# -> Creates ~/.claude/CLOV.md (10 lines, minimal context footprint)
# -> Adds @CLOV.md reference to ~/.claude/CLAUDE.md
# -> Prompts: "Patch settings.json? [y/N]"
# -> If yes: creates backup (~/.claude/settings.json.bak), patches file

# Verify installation
clov init --show  # Shows hook status, settings.json registration
```

**settings.json patching options:**

```bash
clov init -g                 # Default: prompts for consent [y/N]
clov init -g --auto-patch    # Patch immediately without prompting (CI/CD)
clov init -g --no-patch      # Skip patching, print manual JSON snippet
```

**Restart required**: After installation, restart Claude Code, then test with `git status`.

### Manual Install (Fallback)

If automatic patching fails or you want direct control:

```bash
# 1. Install hook and CLOV.md
clov init -g --no-patch  # Prints JSON snippet

# 2. Manually edit ~/.claude/settings.json (add the printed snippet)

# 3. Restart Claude Code
```

**Full manual setup:**

```bash
# 1. Copy the hook script
mkdir -p ~/.claude/hooks
cp .claude/hooks/clov-rewrite.sh ~/.claude/hooks/clov-rewrite.sh
chmod +x ~/.claude/hooks/clov-rewrite.sh

# 2. Add to ~/.claude/settings.json under hooks.PreToolUse:
```

Add this entry to the `PreToolUse` array in `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/clov-rewrite.sh"
          }
        ]
      }
    ]
  }
}
```

### Per-Project Install

The hook lives at `.claude/hooks/clov-rewrite.sh` in this repository. To use it in another project, copy the hook and add the same settings.json entry with a relative path or a project-level `.claude/settings.json`.

### Commands Rewritten

| Raw Command                                                   | Rewritten To             |
| ------------------------------------------------------------- | ------------------------ |
| `git status/diff/log/add/commit/push/pull/branch/fetch/stash` | `clov git ...`           |
| `gh pr/issue/run`                                             | `clov gh ...`            |
| `cargo test/build/clippy`                                     | `clov cargo ...`         |
| `cat <file>`                                                  | `clov read <file>`       |
| `rg/grep <pattern>`                                           | `clov grep <pattern>`    |
| `ls`                                                          | `clov ls`                |
| `vitest/pnpm test`                                            | `clov vitest run`        |
| `tsc/pnpm tsc`                                                | `clov tsc`               |
| `eslint/pnpm lint`                                            | `clov lint`              |
| `prettier`                                                    | `clov prettier`          |
| `playwright`                                                  | `clov playwright`        |
| `prisma`                                                      | `clov prisma`            |
| `ruff check/format`                                           | `clov ruff ...`          |
| `pytest`                                                      | `clov pytest`            |
| `pip list/install/outdated`                                   | `clov pip ...`           |
| `go test/build/vet`                                           | `clov go ...`            |
| `golangci-lint run`                                           | `clov golangci-lint run` |
| `docker ps/images/logs`                                       | `clov docker ...`        |
| `kubectl get/logs`                                            | `clov kubectl ...`       |
| `curl`                                                        | `clov curl`              |
| `pnpm list/ls/outdated`                                       | `clov pnpm ...`          |

Commands already using `clov`, heredocs (`<<`), and unrecognized commands pass through unchanged.

### Suggest Hook (Non-Intrusive Alternative)

If you want Claude Code to suggest clov rather than silently rewriting commands, use the suggest hook instead. It emits a system reminder when clov-compatible commands are detected, without touching the command itself.

**Comparison:**

| Aspect       | Auto-Rewrite Hook                                | Suggest Hook                                                  |
| ------------ | ------------------------------------------------ | ------------------------------------------------------------- |
| **Strategy** | Intercepts and modifies command before execution | Emits system reminder when clov-compatible command detected   |
| **Effect**   | Claude never sees the original command           | Claude receives a hint and decides whether to use clov        |
| **Adoption** | 100% (forced)                                    | ~70-85% (depends on Claude's adherence to instructions)       |
| **Use Case** | Production workflows, guaranteed savings         | Learning mode, auditing, user preference for explicit control |
| **Overhead** | Zero (transparent rewrite)                       | Minimal (reminder message in context)                         |

When to use suggest instead of rewrite:

- You want to audit which commands Claude chooses to run
- You are learning clov and want to see what the rewrite logic does
- You prefer Claude to make explicit decisions rather than silent ones
- You need exact command execution for debugging

**Setup:**

```bash
mkdir -p ~/.claude/hooks
cp .claude/hooks/clov-suggest.sh ~/.claude/hooks/clov-suggest.sh
chmod +x ~/.claude/hooks/clov-suggest.sh
```

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/clov-suggest.sh"
          }
        ]
      }
    ]
  }
}
```

The suggest hook detects the same commands as the rewrite hook but outputs a `systemMessage` instead of `updatedInput`, so Claude receives a hint rather than a rewritten command.

## Uninstalling clov

**Complete removal (global):**

```bash
clov init -g --uninstall

# Removes:
#   - ~/.claude/hooks/clov-rewrite.sh
#   - ~/.claude/CLOV.md
#   - @CLOV.md reference from ~/.claude/CLAUDE.md
#   - clov hook entry from ~/.claude/settings.json

# Restart Claude Code after uninstall
```

**Restore from backup:**

```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

**Local projects**: Manually remove clov instructions from `./CLAUDE.md`

**Binary removal:**

```bash
# Installed via cargo
cargo uninstall clov

# Installed via package manager
brew uninstall clov          # macOS Homebrew
sudo apt remove clov         # Debian/Ubuntu
sudo dnf remove clov         # Fedora/RHEL
```

## Documentation

- **[INSTALL.md](INSTALL.md)**: Detailed installation guide with verification steps
- **[AUDIT_GUIDE.md](docs/AUDIT_GUIDE.md)**: Guide to token savings analytics, temporal breakdowns, and data export
- **[CLAUDE.md](CLAUDE.md)**: Claude Code integration instructions and project context
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: Technical architecture and development guide

## Troubleshooting

### settings.json Patching Failed

**Problem**: `clov init -g` fails to patch settings.json

```bash
# Check if settings.json is valid JSON
cat ~/.claude/settings.json | python3 -m json.tool

# Use manual patching
clov init -g --no-patch  # Prints JSON snippet

# Restore from backup
cp ~/.claude/settings.json.bak ~/.claude/settings.json

# Check permissions
ls -la ~/.claude/settings.json
chmod 644 ~/.claude/settings.json
```

### Hook Not Working After Install

**Problem**: Commands are not going through clov after `clov init -g`

```bash
# Verify hook is registered
clov init --show

# Check settings.json manually
cat ~/.claude/settings.json | grep clov-rewrite

# Restart Claude Code (required step)

# Test with a command
git status  # Should use clov automatically
```

### Uninstall Did Not Remove Everything

**Problem**: clov traces remain after `clov init -g --uninstall`

```bash
# Remove hook
rm ~/.claude/hooks/clov-rewrite.sh

# Remove CLOV.md
rm ~/.claude/CLOV.md

# Remove @CLOV.md reference
nano ~/.claude/CLAUDE.md  # Delete @CLOV.md line

# Remove from settings.json
nano ~/.claude/settings.json  # Remove clov hook entry

# Restore from backup
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```


## For Maintainers

### Security Review Workflow

clov runs a 3-layer security review on external PRs.

**Layer 1: Automated GitHub Action**

Every PR triggers `.github/workflows/security-check.yml`:

- Cargo audit: CVE detection in dependencies
- Critical files alert: flags modifications to high-risk files (runner.rs, tracking.rs, Cargo.toml, workflows)
- Dangerous pattern scanning: shell injection, network operations, unsafe code, panic risks
- Dependency auditing: supply chain verification for new crates
- Clippy security lints: enforces Rust safety best practices

Results appear in the PR's GitHub Actions summary.

**Layer 2: Claude Code Skill**

Maintainers with [Claude Code](https://claude.ai/code) can run:

```bash
/clov-pr-security <PR_NUMBER>
```

The skill performs:

- Critical files analysis: detects modifications to shell execution, validation, or CI/CD files
- Dangerous pattern detection: identifies shell injection, environment manipulation, exfiltration vectors
- Supply chain audit: verifies new dependencies on crates.io (downloads, maintainer, license)
- Semantic analysis: checks intent vs reality, logic bombs, code quality red flags
- Structured report: produces security assessment with risk level and verdict

**Skill installation (maintainers only):**

```bash
cp -r ~/.claude/skills/clov-pr-security ~/.claude/skills/
```

The skill includes:

- `SKILL.md`: workflow automation and usage guide
- `critical-files.md`: clov-specific file risk tiers with attack scenarios
- `dangerous-patterns.md`: regex patterns with exploitation examples
- `checklist.md`: manual review template

**Layer 3: Manual Review**

For PRs that touch critical files or add dependencies:

- 2 maintainers required for Cargo.toml, workflows, or Tier 1 files
- Isolated testing recommended for high-risk changes

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Open an issue or PR on GitHub.

External contributors: your PR goes through automated security review. This protects clov's shell execution capabilities against injection attacks and supply chain vulnerabilities.

## Contact

- Issues: https://github.com/alexandephilia/clov-ai/issues
- Email: 4lexander31@gmail.com
