# clov — Clov Token Omitter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A high-performance CLI proxy that slashes LLM token consumption by 60–90%.**

![clov preview](img_1.jpg)

clov sits between your shell and LLM context, intercepting command output and compressing it through smart filtering, grouping, truncation, and deduplication — so your agent spends tokens on reasoning, not noise.

## Token Savings — 30-Min Claude Code Session

![clov preview 2](img_2.jpg)

Typical session without clov: **~150,000 tokens**
With clov: **~45,000 tokens** → **70% reduction**

| Operation                 | Frequency | Standard     | clov        | Savings  |
| ------------------------- | --------- | ------------ | ----------- | -------- |
| `ls` / `tree`             | 10×       | 2,000        | 400         | -80%     |
| `cat` / `read`            | 20×       | 40,000       | 12,000      | -70%     |
| `grep` / `rg`             | 8×        | 16,000       | 3,200       | -80%     |
| `git status`              | 10×       | 3,000        | 600         | -80%     |
| `git diff`                | 5×        | 10,000       | 2,500       | -75%     |
| `git log`                 | 5×        | 2,500        | 500         | -80%     |
| `git add/commit/push`     | 8×        | 1,600        | 120         | -92%     |
| `npm test` / `cargo test` | 5×        | 25,000       | 2,500       | -90%     |
| `ruff check`              | 3×        | 3,000        | 600         | -80%     |
| `pytest`                  | 4×        | 8,000        | 800         | -90%     |
| `go test`                 | 3×        | 6,000        | 600         | -90%     |
| `docker ps`               | 3×        | 900          | 180         | -80%     |
| **Total**                 |           | **~118,000** | **~23,900** | **-80%** |

> Estimates based on medium-sized TypeScript/Rust projects. Actual savings vary by project size.

## Installation

### ⚠️ Pre-Installation Check (REQUIRED)

**Always verify whether clov is already installed before proceeding:**

```bash
clov --version        # Check if installed
clov gain             # Verify it's the Token Omitter
which clov            # Check installation path
```

If already installed and `clov gain` works, **do not reinstall**. Skip to Quick Start.

### Homebrew (macOS/Linux)

```bash
brew tap alexandephilia/clov
brew install clov
```

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/alexandephilia/clov-ai/refs/heads/main/install.sh | sh
```

> **Note**: clov installs to `~/.local/bin` by default. If this directory is not in your PATH, add it:
>
> ```bash
> echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
> ```

After installation, **confirm clov is operational**:

```bash
clov gain  # Must display token savings stats — not "command not found"
```

### Alternative: Manual Installation

```bash
cargo install --git https://github.com/alexandephilia/clov-ai

```

### Alternative: Pre-built Binaries

Download from [clov-ai/releases](https://github.com/alexandephilia/clov-ai/releases):

- macOS: `clov-x86_64-apple-darwin.tar.gz` / `clov-aarch64-apple-darwin.tar.gz`
- Linux: `clov-x86_64-unknown-linux-gnu.tar.gz` / `clov-aarch64-unknown-linux-gnu.tar.gz`
- Windows: `clov-x86_64-pc-windows-msvc.zip`

## Quick Start

```bash
# 1. Verify installation
clov gain  # Must show token stats, not "command not found"

# 2. Initialize for Claude Code (RECOMMENDED: hook-first mode)
clov init --global
# → Installs hook + creates slim CLOV.md (10 lines, 99.5% token savings)
# → Follow printed instructions to add hook to ~/.claude/settings.json

# 3. Test it works
clov git status  # Should show ultra-compact output
clov init --show # Verify hook is installed and executable

# Alternative modes:
# clov init --global --claude-md  # Legacy: full injection (137 lines)
# clov init                       # Local project only (./CLAUDE.md)
```

**New in v0.9.5**: Hook-first installation eliminates ~2,000 tokens from Claude's context while preserving full clov functionality through transparent command rewriting.

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
clov read file.rs -l aggressive  # Signatures only (strips bodies)
clov smart file.rs               # 2-line heuristic code summary
clov find "*.rs" .               # Compact find results
clov grep "pattern" .            # Grouped search results
```

### Git

```bash
clov git status                  # Compact status
clov git log -n 10               # One-line commits
clov git diff                    # Condensed diff
clov git add                     # → "ok ✓"
clov git commit -m "msg"         # → "ok ✓ abc1234"
clov git push                    # → "ok ✓ main"
clov git pull                    # → "ok ✓ 3 files +10 -2"
```

### Commands

```bash
clov test cargo test             # Show failures only (-90% tokens)
clov err npm run build           # Errors/warnings only
clov summary <long command>      # Heuristic summary
clov log app.log                 # Deduplicated logs
clov gh pr list                   # Compact PR listing
clov gh pr view 42                # PR details + checks summary
clov gh issue list                # Compact issue listing
clov gh run list                  # Workflow run status
clov wget https://example.com    # Download, strip progress bars
clov config                       # Show config (--create to generate)
clov ruff check                   # Python linting (JSON, 80% reduction)
clov pytest                       # Python tests (failures only, 90% reduction)
clov pip list                     # Python packages (auto-detect uv, 70% reduction)
clov go test                      # Go tests (NDJSON, 90% reduction)
clov golangci-lint run            # Go linting (JSON, 85% reduction)
```

### Data & Analytics

```bash
clov json config.json            # Structure without values
clov deps                        # Dependencies summary
clov env -f AWS                  # Filtered env vars

# Token Savings Analytics (includes execution time metrics)
clov gain                        # Summary stats with total exec time
clov gain --graph                # With ASCII graph of last 30 days
clov gain --history              # With recent command history (10)
clov gain --quota --tier 20x     # Monthly quota analysis (pro/5x/20x)

# Temporal Breakdowns (includes time metrics per period)
clov gain --daily                # Day-by-day with avg execution time
clov gain --weekly               # Week-by-week breakdown
clov gain --monthly              # Month-by-month breakdown
clov gain --all                  # All breakdowns combined

# Export Formats (includes total_time_ms and avg_time_ms fields)
clov gain --all --format json    # JSON export for APIs/dashboards
clov gain --all --format csv     # CSV export for Excel/analysis
```

> 📖 **API Documentation**: For programmatic access to tracking data (Rust library usage, CI/CD integration, custom dashboards), see [docs/tracking.md](docs/tracking.md).

### Discover — Find Missed Savings

Scans your Claude Code session history to find commands where clov would have saved tokens. Use it to:

- **Measure what you're missing** — see exactly how many tokens you could save
- **Identify habits** — find which commands you keep running without clov
- **Spot new opportunities** — see unhandled commands that could become clov features

```bash
clov discover                    # Current project, last 30 days
clov discover --all              # All Claude Code projects
clov discover --all --since 7    # Last 7 days across all projects
clov discover -p aristote        # Filter by project name (substring)
clov discover --format json      # Machine-readable output
```

Example output:

```
CLOV Discover -- Savings Opportunities
====================================================
Scanned: 142 sessions (last 30 days), 1786 Bash commands
Already using CLOV: 108 commands (6%)

MISSED SAVINGS -- Commands CLOV already handles
----------------------------------------------------
Command              Count    CLOV Equivalent        Est. Savings
git log                434    clov git               ~55.9K tokens
cargo test             203    clov cargo             ~49.9K tokens
ls -la                 107    clov ls                ~11.8K tokens
gh pr                   80    clov gh                ~10.4K tokens
----------------------------------------------------
Total: 986 commands -> ~143.9K tokens saveable

TOP UNHANDLED COMMANDS -- open an issue?
----------------------------------------------------
Command              Count    Example
git checkout            84    git checkout feature/my-branch
cargo run               32    cargo run -- gain --help
----------------------------------------------------
-> github.com/alexandephilia/clov-ai/issues
```

### Containers

```bash
clov docker ps                   # Compact container list
clov docker images               # Compact image list
clov docker logs <container>     # Deduplicated logs
clov kubectl pods                # Compact pod list
clov kubectl logs <pod>          # Deduplicated logs
clov kubectl services             # Compact service list
```

### JavaScript / TypeScript Stack

```bash
clov lint                         # ESLint grouped by rule/file
clov lint biome                   # Supports other linters too
clov tsc                          # TypeScript errors grouped by file
clov next build                   # Next.js build compact output
clov prettier --check .           # Files needing formatting
clov vitest run                   # Test failures only
clov playwright test              # E2E results (failures only)
clov prisma generate              # Schema generation (no ASCII art)
clov prisma migrate dev --name x  # Migration summary
clov prisma db-push               # Schema push summary
```

### Python & Go Stack

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

### Standard vs clov

**Directory listing:**

```
# ls -la (45 lines, ~800 tokens)
drwxr-xr-x  15 user  staff    480 Jan 23 10:00 .
drwxr-xr-x   5 user  staff    160 Jan 23 09:00 ..
-rw-r--r--   1 user  staff   1234 Jan 23 10:00 Cargo.toml
...

# clov ls (12 lines, ~150 tokens)
📁 my-project/
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
  ╭─────────────────────────────────────────────────────────────────╮
  │  WITHOUT CLOV                                                   │
  ├─────────────────────────────────────────────────────────────────┤
  │                                                                 │
  │  ┌──────────┐   git status   ┌──────────┐   git status   ┌──────────┐
  │  │  Claude  │ ─────────────► │  shell   │ ─────────────► │   git    │
  │  │   LLM    │                │          │                │  (CLI)   │
  │  └──────────┘                └──────────┘                └────┬─────┘
  │        ▲                                                       │      │
  │        │                ~2,000 tokens  (raw, verbose output)   │      │
  │        └───────────────────────────────────────────────────────┘      │
  │                                                                 │
  ╰─────────────────────────────────────────────────────────────────╯

  ╭─────────────────────────────────────────────────────────────────╮
  │  WITH CLOV                                                      │
  ├─────────────────────────────────────────────────────────────────┤
  │                                                                 │
  │  ┌──────────┐   git status   ┌──────────┐   git status   ┌──────────┐
  │  │  Claude  │ ─────────────► │   clov   │ ─────────────► │   git    │
  │  │   LLM    │                │  (proxy) │                │  (CLI)   │
  │  └──────────┘                └────┬─────┘                └────┬─────┘
  │        ▲                          │    raw output (~2,000 tok) │      │
  │        │                          └───────────────────────────┘      │
  │        │  ~200 tokens (filtered)                                      │
  │        │  filter · group · dedup · truncate                          │
  │        └───────────────────────────────────────────────────────────── │
  │                                                                 │
  ╰─────────────────────────────────────────────────────────────────╯
```

Four strategies applied per command type:

1. **Smart Filtering**: Removes noise (comments, whitespace, boilerplate)
2. **Grouping**: Aggregates similar items (files by directory, errors by type)
3. **Truncation**: Keeps relevant context, cuts redundancy
4. **Deduplication**: Collapses repeated log lines with counts

## Configuration

### Installation Modes

| Command                    | Scope  | Hook | CLOV.md       | CLAUDE.md        | Tokens in Context | Use Case                                 |
| -------------------------- | ------ | ---- | ------------- | ---------------- | ----------------- | ---------------------------------------- |
| `clov init -g`             | Global | ✅   | ✅ (10 lines) | @CLOV.md         | ~10               | **Recommended**: All projects, automatic |
| `clov init -g --claude-md` | Global | ❌   | ❌            | Full (137 lines) | ~2000             | Legacy compatibility                     |
| `clov init -g --hook-only` | Global | ✅   | ❌            | Nothing          | 0                 | Minimal setup, hook-only                 |
| `clov init`                | Local  | ❌   | ❌            | Full (137 lines) | ~2000             | Single project, no hook                  |

```bash
clov init --show         # Show current configuration
clov init -g             # Install hook + CLOV.md (recommended)
clov init -g --claude-md # Legacy: full injection into CLAUDE.md
clov init                # Local project: full injection into ./CLAUDE.md
```

### Installation Flags

**Settings.json Control**:

```bash
clov init -g                 # Default: prompt to patch [y/N]
clov init -g --auto-patch    # Patch settings.json without prompting
clov init -g --no-patch      # Skip patching, show manual instructions
```

**Mode Control**:

```bash
clov init -g --claude-md     # Legacy: full 137-line injection (no hook)
clov init -g --hook-only     # Hook only, no CLOV.md
```

**Uninstall**:

```bash
clov init -g --uninstall     # Remove all CLOV artifacts
```

**What is settings.json?**
Claude Code configuration file that registers the CLOV hook. The hook transparently rewrites commands (e.g., `git status` → `clov git status`) before execution. Without this registration, Claude won't use the hook.

**Backup Behavior**:
CLOV creates `~/.claude/settings.json.bak` before making changes. If something breaks, restore with:

```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

**Migration**: If you previously used `clov init -g` with the old system (137-line injection), simply re-run `clov init -g` to automatically migrate to the new hook-first approach.

**Real-session example — 3 days of tracked usage:**

```bash
📊 CLOV Token Savings
════════════════════════════════════════

Total commands:    133
Input tokens:      30.5K
Output tokens:     10.7K
Tokens saved:      25.3K (83.0%)

By Command:
────────────────────────────────────────
Command               Count      Saved     Avg%
clov git status           41      17.4K    82.9%
clov git push             54       3.4K    91.6%
clov grep                 15       3.2K    26.5%
clov ls                   23       1.4K    37.2%

Daily Savings (last 30 days):
────────────────────────────────────────
01-23 │███████████████████                      6.4K
01-24 │██████████████████                       5.9K
01-25 │                                         18
01-26 │████████████████████████████████████████ 13.0K
```

### Custom Database Path

By default, CLOV stores tracking data in `~/.local/share/clov/history.db`. You can override this:

**Environment variable** (highest priority):

```bash
export CLOV_DB_PATH="/path/to/custom.db"
```

**Config file** (`~/.config/clov/config.toml`):

```toml
[tracking]
database_path = "/path/to/custom.db"
```

Priority: `CLOV_DB_PATH` env var > `config.toml` > default location.

### Tee: Full Output Recovery

When CLOV filters command output, LLM agents lose failure details (stack traces, assertion messages) and may re-run the same command 2-3 times. The **tee** feature saves raw output to a file so the agent can read it without re-executing.

**How it works**: On command failure, CLOV writes the full unfiltered output to `~/.local/share/clov/tee/` and prints a one-line hint:

```
✓ cargo test: 15 passed (1 suite, 0.01s)
[full output: ~/.local/share/clov/tee/1707753600_cargo_test.log]
```

The agent reads the file instead of re-running the command — saving tokens.

**Default behavior**: Tee only on failures (exit code != 0), skip outputs < 500 chars.

**Config** (`~/.config/clov/config.toml`):

```toml
[tee]
enabled = true          # default: true
mode = "failures"       # "failures" (default), "always", or "never"
max_files = 20          # max files to keep (oldest rotated out)
max_file_size = 1048576 # 1MB per file max
# directory = "/custom/path"  # override default location
```

**Environment overrides**:

- `CLOV_TEE=0` — disable tee entirely
- `CLOV_TEE_DIR=/path` — override output directory

**Supported commands**: cargo (build/test/clippy/check/install/nextest), vitest, pytest, lint (eslint/biome/ruff/pylint/mypy), tsc, go (test/build/vet), err, test.

## Auto-Rewrite Hook (Recommended)

The most effective way to deploy clov is via the **auto-rewrite hook** for Claude Code. Rather than relying on CLAUDE.md instructions — which subagents may disregard — this hook transparently intercepts Bash commands and rewrites them to their clov equivalents before execution.

**Result**: 100% clov adoption across all conversations and subagents, zero token overhead in Claude's context.

### What Are Hooks?

**For the uninitiated**:
Claude Code hooks are scripts that execute before or after Claude runs commands. clov uses a **PreToolUse** hook that intercepts Bash commands and rewrites them (e.g., `git status` → `clov git status`) before they reach the shell. The rewrite is entirely transparent — Claude never observes it; it simply receives optimized output.

**Why settings.json?**
Claude Code reads `~/.claude/settings.json` to discover registered hooks. Without this file, Claude has no awareness that the clov hook exists. Think of it as the hook registry.

**Is it safe?**
Yes. clov creates a backup (`settings.json.bak`) before any modifications. The hook is strictly read-only — it rewrites command strings only, and never touches files or credentials. Inspect the hook script at `~/.claude/hooks/clov-rewrite.sh` at any time.

### How It Works

The hook runs as a Claude Code [PreToolUse hook](https://docs.anthropic.com/en/docs/claude-code/hooks). When Claude Code is about to execute a Bash command like `git status`, the hook rewrites it to `clov git status` before it reaches the shell. The rewrite is invisible to Claude — it simply receives filtered, token-efficient output.

```
  Claude types:  git status
                      │
                      ▼
         ┌────────────────────────────┐
         │  ~/.claude/settings.json   │
         │  PreToolUse hook           │
         │  registered                │
         └────────────┬───────────────┘
                      │
                      ▼
         ┌────────────────────────────┐
         │  clov-rewrite.sh           │
         │                            │
         │  "git status"              │
         │       ──────────────►      │  ← transparent rewrite
         │  "clov git status"         │
         └────────────┬───────────────┘
                      │
                      ▼
         ┌────────────────────────────┐
         │  clov  (Rust binary)       │
         │  · runs real git           │
         │  · filters output          │
         │  · tracks token savings    │
         └────────────┬───────────────┘
                      │
                      ▼
  Claude receives:  "3 modified, 1 untracked ✓"
                    (not 50 lines of raw git output)
```

### Quick Install (Automated)

```bash
clov init -g
# → Installs hook to ~/.claude/hooks/clov-rewrite.sh (with executable permissions)
# → Creates ~/.claude/CLOV.md (10 lines, minimal context footprint)
# → Adds @CLOV.md reference to ~/.claude/CLAUDE.md
# → Prompts: "Patch settings.json? [y/N]"
# → If yes: creates backup (~/.claude/settings.json.bak), patches file

# Verify installation
clov init --show  # Shows hook status, settings.json registration
```

**Settings.json Patching Options**:

```bash
clov init -g                 # Default: prompts for consent [y/N]
clov init -g --auto-patch    # Patch immediately without prompting (CI/CD)
clov init -g --no-patch      # Skip patching, print manual JSON snippet
```

**What is settings.json?**
Claude Code's configuration file that registers the CLOV hook. Without this, Claude won't use the hook. CLOV backs up the file before changes (`settings.json.bak`).

**Restart Required**: After installation, restart Claude Code, then test with `git status`.

### Manual Install (Fallback)

If automatic patching fails or you prefer manual control:

```bash
# 1. Install hook and CLOV.md
clov init -g --no-patch  # Prints JSON snippet

# 2. Manually edit ~/.claude/settings.json (add the printed snippet)

# 3. Restart Claude Code
```

**Alternative: Full manual setup**

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

The hook is included in this repository at `.claude/hooks/clov-rewrite.sh`. To use it in another project, copy the hook and add the same settings.json entry using a relative path or project-level `.claude/settings.json`.

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

### Alternative: Suggest Hook (Non-Intrusive)

If you prefer Claude Code to **suggest** clov usage rather than automatically rewriting commands, use the **suggest hook** pattern instead. This emits a system reminder when clov-compatible commands are detected, without modifying the command execution.

**Comparison**:

| Aspect       | Auto-Rewrite Hook                                | Suggest Hook                                                  |
| ------------ | ------------------------------------------------ | ------------------------------------------------------------- |
| **Strategy** | Intercepts and modifies command before execution | Emits system reminder when clov-compatible command detected   |
| **Effect**   | Claude Code never sees the original command      | Claude Code receives hint to use clov, decides autonomously   |
| **Adoption** | 100% (forced)                                    | ~70-85% (depends on Claude Code's adherence to instructions)  |
| **Use Case** | Production workflows, guaranteed savings         | Learning mode, auditing, user preference for explicit control |
| **Overhead** | Zero (transparent rewrite)                       | Minimal (reminder message in context)                         |

**When to use suggest over rewrite**:

- You want to audit which commands Claude Code chooses to run
- You're learning clov patterns and want visibility into the rewrite logic
- You prefer Claude Code to make explicit decisions rather than transparent rewrites
- You want to preserve exact command execution for debugging

#### Suggest Hook Setup

**1. Create the suggest hook script**

```bash
mkdir -p ~/.claude/hooks
cp .claude/hooks/clov-suggest.sh ~/.claude/hooks/clov-suggest.sh
chmod +x ~/.claude/hooks/clov-suggest.sh
```

**2. Add to `~/.claude/settings.json`**

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

The suggest hook detects the same commands as the rewrite hook but emits a `systemMessage` instead of `updatedInput`, informing Claude Code that a clov alternative is available — without modifying the command itself.

## Uninstalling CLOV

**Complete Removal (Global Only)**:

```bash
clov init -g --uninstall

# Removes:
#   - ~/.claude/hooks/clov-rewrite.sh
#   - ~/.claude/CLOV.md
#   - @CLOV.md reference from ~/.claude/CLAUDE.md
#   - CLOV hook entry from ~/.claude/settings.json

# Restart Claude Code after uninstall
```

**Restore from Backup** (if needed):

```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

**Local Projects**: Manually remove CLOV instructions from `./CLAUDE.md`

**Binary Removal**:

```bash
# If installed via cargo
cargo uninstall clov

# If installed via package manager
brew uninstall clov          # macOS Homebrew
sudo apt remove clov         # Debian/Ubuntu
sudo dnf remove clov         # Fedora/RHEL
```

## Documentation

- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - ⚠️ Fix common issues (wrong clov installed, missing commands, PATH issues)
- **[INSTALL.md](INSTALL.md)** - Detailed installation guide with verification steps
- **[AUDIT_GUIDE.md](docs/AUDIT_GUIDE.md)** - Complete guide to token savings analytics, temporal breakdowns, and data export
- **[CLAUDE.md](CLAUDE.md)** - Claude Code integration instructions and project context
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Technical architecture and development guide
- **[SECURITY.md](SECURITY.md)** - Security policy, vulnerability reporting, and PR review process

## Troubleshooting

### Settings.json Patching Failed

**Problem**: `clov init -g` fails to patch settings.json

**Solutions**:

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

**Problem**: Commands still not using CLOV after `clov init -g`

**Solutions**:

```bash
# Verify hook is registered
clov init --show

# Check settings.json manually
cat ~/.claude/settings.json | grep clov-rewrite

# Restart Claude Code (critical step!)

# Test with a command
git status  # Should use clov automatically
```

### Uninstall Didn't Remove Everything

**Problem**: CLOV traces remain after `clov init -g --uninstall`

**Manual Cleanup**:

```bash
# Remove hook
rm ~/.claude/hooks/clov-rewrite.sh

# Remove CLOV.md
rm ~/.claude/CLOV.md

# Remove @CLOV.md reference
nano ~/.claude/CLAUDE.md  # Delete @CLOV.md line

# Remove from settings.json
nano ~/.claude/settings.json  # Remove CLOV hook entry

# Restore from backup
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

See **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** for more issues and solutions.

## For Maintainers

### Security Review Workflow

CLOV implements a comprehensive 3-layer security review process for external PRs:

#### Layer 1: Automated GitHub Action

Every PR triggers `.github/workflows/security-check.yml`:

- **Cargo audit**: CVE detection in dependencies
- **Critical files alert**: Flags modifications to high-risk files (runner.rs, tracking.rs, Cargo.toml, workflows)
- **Dangerous pattern scanning**: Shell injection, network operations, unsafe code, panic risks
- **Dependency auditing**: Supply chain verification for new crates
- **Clippy security lints**: Enforces Rust safety best practices

Results appear in the PR's GitHub Actions summary.

#### Layer 2: Claude Code Skill

For comprehensive manual review, maintainers with [Claude Code](https://claude.ai/code) can use:

```bash
/clov-pr-security <PR_NUMBER>
```

The skill performs:

- **Critical files analysis**: Detects modifications to shell execution, validation, or CI/CD files
- **Dangerous pattern detection**: Identifies shell injection, environment manipulation, exfiltration vectors
- **Supply chain audit**: Verifies new dependencies on crates.io (downloads, maintainer, license)
- **Semantic analysis**: Checks intent vs reality, logic bombs, code quality red flags
- **Structured report generation**: Produces security assessment with risk level and verdict

**Skill installation** (maintainers only):

```bash
# The skill is bundled in the clov-pr-security directory
# Copy to your Claude skills directory:
cp -r ~/.claude/skills/clov-pr-security ~/.claude/skills/
```

The skill includes:

- `SKILL.md` - Workflow automation and usage guide
- `critical-files.md` - CLOV-specific file risk tiers with attack scenarios
- `dangerous-patterns.md` - Regex patterns with exploitation examples
- `checklist.md` - Manual review template

#### Layer 3: Manual Review

For PRs touching critical files or adding dependencies:

- **2 maintainers required** for Cargo.toml, workflows, or Tier 1 files
- **Isolated testing** recommended for high-risk changes
- Follow the checklist in SECURITY.md

See **[SECURITY.md](SECURITY.md)** for complete security policy and review guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome — open an issue to discuss ideas or submit a PR directly.

**For external contributors**: Your PR will undergo automated security review (see [SECURITY.md](SECURITY.md)). This protects CLOV's shell execution capabilities against injection attacks and supply chain vulnerabilities.

## Contact

- Issues: https://github.com/alexandephilia/clov-ai/issues
