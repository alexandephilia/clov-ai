# clov

<p align="center">
  <img src="assets/clov_mascot.png" width="400" alt="clov mascot">
</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.26.5-blue.svg)](https://github.com/alexandephilia/clov-ai/releases/tag/v0.26.5)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-supported-3776AB?logo=python&logoColor=white)](#python--go)
[![Go](https://img.shields.io/badge/Go-supported-00ADD8?logo=go&logoColor=white)](#python--go)
[![Claude Code](https://img.shields.io/badge/Claude_Code-integrated-7B2D8B?logo=anthropic&logoColor=white)](https://claude.ai/code)

**A shell filter that keeps token costs sane.**

![clov preview](clov_1.jpg)

Your shell dumps thousands of tokens into Claude's context every session. Progress bars. ANSI codes. Timestamps. Verbose git output nobody asked for. clov intercepts that garbage before it reaches the model and hands back only what matters — hashes, failures, errors. One install. Every session.

---

## Why

![clov savings](clov_2.jpg)

A medium session without clov burns around **150,000 tokens**. With clov: roughly **45,000**. That's a real number, not marketing.

| Command                        | Raw tokens   | With clov   | Cut     |
| ------------------------------ | ------------ | ----------- | ------- |
| `ls` / `tree` (×10)            | 2,000        | 400         | 80%     |
| `cat` / file reads (×20)       | 40,000       | 12,000      | 70%     |
| `grep` / `rg` (×8)             | 16,000       | 3,200       | 80%     |
| `git status` (×10)             | 3,000        | 600         | 80%     |
| `git diff` (×5)                | 10,000       | 2,500       | 75%     |
| `git log` (×5)                 | 2,500        | 500         | 80%     |
| `git add/commit/push` (×8)     | 1,600        | 120         | 92%     |
| `cargo test` / `npm test` (×5) | 25,000       | 2,500       | 90%     |
| `ruff check` (×3)              | 3,000        | 600         | 80%     |
| `pytest` (×4)                  | 8,000        | 800         | 90%     |
| `go test` (×3)                 | 6,000        | 600         | 90%     |
| `docker ps` (×3)               | 900          | 180         | 80%     |
| **Total**                      | **~118,000** | **~23,900** | **80%** |

> Based on medium TypeScript/Rust projects. Actual savings vary.

---

## Install

### Check first (seriously)

If `clov` is already on your machine:

```bash
clov --version   # installed?
clov gain        # is it actually clov, or some other binary?
which clov       # where does it live?
```

If `clov gain` prints stats, you are done. Go to [Setup](#setup).

### Homebrew

```bash
brew tap alexandephilia/clov
brew install clov
```

### Curl installer

```bash
curl -fsSL https://raw.githubusercontent.com/alexandephilia/clov-ai/refs/heads/main/install.sh | sh
```

Installs to `~/.local/bin`. If that is not in your PATH:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc  # or ~/.bashrc
```

Then verify:

```bash
clov gain   # must print stats, not "command not found"
```

### Cargo

```bash
cargo install --git https://github.com/alexandephilia/clov-ai
```

### Pre-built binaries

Grab from [releases](https://github.com/alexandephilia/clov-ai/releases):

- macOS: `clov-x86_64-apple-darwin.tar.gz` / `clov-aarch64-apple-darwin.tar.gz`
- Linux: `clov-x86_64-unknown-linux-gnu.tar.gz` / `clov-aarch64-unknown-linux-gnu.tar.gz`
- Windows: `clov-x86_64-pc-windows-msvc.zip`

---

## Setup

```bash
# Hook-first install (recommended)
clov init --global
# Installs the rewrite hook + a 10-line CLOV.md reference
# Follow the printed instructions to register it in ~/.claude/settings.json

# Confirm it is wired up
clov init --show
clov git status   # should produce compact output

# Alternative modes:
clov init --global --claude-md  # full 137-line injection into CLAUDE.md (legacy)
clov init                       # local project only
```

**v0.25.0**: The hook-first mode removes ~2,000 tokens of setup boilerplate from Claude's context while keeping full coverage through automatic command rewriting.

---

## How It Works

```
without clov:   model <──────── shell output  (~2,000 tokens)

with clov:      model <── clov <── shell output  (~200 tokens)
                           filter
```

Four strategies:

1. **Filtering** — strips ANSI codes, blank lines, boilerplate, progress noise
2. **Grouping** — collapses related items (errors by file, packages by type)
3. **Truncation** — keeps context, cuts repetition
4. **Deduplication** — repeated log lines become single entries with counts

---

## Global Flags

```bash
-u, --ultra-compact   # icon-only format, maximum savings
-v, --verbose         # show more (-v, -vv, -vvv)
```

---

## Commands

### Files

```bash
clov ls .                        # compact directory tree
clov tree .                      # tree view
clov read file.rs                # filtered file read
clov read file.rs -l aggressive  # signatures only, strips bodies
clov smart file.rs               # 2-line heuristic summary
clov find "*.rs" .               # compact find output
clov grep "pattern" .            # grouped by file
```

### Git

```bash
clov git status                  # compact: "3 modified, 1 untracked"
clov git log -n 10               # one line per commit
clov git diff                    # condensed diff
clov git add .                   # -> "ok ✓"
clov git commit -m "msg"         # -> "ok ✓ abc1234"
clov git push                    # -> "ok ✓ main"
clov git pull                    # -> "ok ✓ 3 files +10 -2"
clov git stash                   # compact stash output
clov git worktree list           # compact worktree list
```

### Graphite (gt)

Stacked PR workflows with the same token-optimized output:

```bash
clov gt log                      # compact stack graph, strips emails
clov gt log short                # short format passthrough
clov gt submit                   # push summary: "pushed feat/x, created PR #42"
clov gt sync                     # sync summary: "ok sync: 2 synced, 1 deleted"
clov gt restack                  # restack summary: "ok restacked 3 branches"
clov gt create                   # create summary: "ok created feat/new-feature"
clov gt branch                   # branch info
clov gt status                   # routes through git filter
```

### Test & Build

```bash
clov test cargo test             # failures only (90% reduction)
clov err npm run build           # errors and warnings only
clov cargo test                  # compact cargo test output
clov cargo build                 # errors only on failure
clov cargo clippy                # grouped lint output
```

### GitHub CLI

```bash
clov gh pr list                  # compact PR table
clov gh pr view 42               # PR details + check summary
clov gh issue list               # compact issue table
clov gh run list                 # workflow run status
```

### JavaScript / TypeScript

```bash
clov lint                        # ESLint grouped by rule/file (84% reduction)
clov lint biome                  # works with Biome too
clov tsc                         # TypeScript errors grouped by file (83% reduction)
clov next build                  # Next.js build metrics only (87% reduction)
clov prettier --check .          # files that need formatting (70% reduction)
clov vitest run                  # failures only (99.5% reduction)
clov playwright test             # E2E failures only (94% reduction)
clov prisma generate             # schema output without ASCII art (88% reduction)
clov prisma migrate dev --name x
clov prisma db-push
clov npm run build               # strips npm boilerplate
clov npx tsc                     # routes through tsc filter
clov pnpm list                   # compact dependency tree (70-90% reduction)
clov pnpm outdated
clov pnpm install
```

### Python / Go

```bash
# Python
clov ruff check                  # JSON output, grouped (80% reduction)
clov ruff format                 # files changed only
clov pytest                      # failures only, state machine parser (90% reduction)
clov mypy                        # grouped by file/error code (80% reduction)
clov pip list                    # auto-detects uv (70% reduction)
clov pip install <pkg>
clov pip outdated                # (85% reduction)

# Go
clov go test                     # NDJSON streaming (90% reduction)
clov go build                    # errors only (80% reduction)
clov go vet                      # vet issues (75% reduction)
clov golangci-lint run           # JSON grouped by rule (85% reduction)
```

### Infrastructure

```bash
clov docker ps                   # compact container list
clov docker images               # compact image list
clov docker logs <container>     # deduplicated
clov docker compose up
clov kubectl pods                # compact pod list
clov kubectl logs <pod>          # deduplicated
clov kubectl services
clov aws sts get-caller-identity # force JSON, compress
clov psql <args>                 # strip borders, compact tables
clov curl <url>                  # auto-JSON detection, schema output
```

### Data

```bash
clov json config.json            # structure without values
clov json config.json -d 3      # depth limit
clov deps                        # dependency summary
clov env -f AWS                  # filtered env vars (sensitive masked)
clov log app.log                 # deduplicated log stream
clov wc file.txt                 # compact word/line/byte count
clov wget <url>                  # download, strips progress bars
clov diff file1 file2            # changed lines only
clov summary <cmd>               # run command, show heuristic summary
```

### Analytics

```bash
clov gain                        # session summary + total exec time
clov gain --graph                # ASCII savings graph, last 30 days
clov gain --history              # recent 10 commands
clov gain --quota --tier 20x     # monthly quota analysis (pro/5x/20x)
clov gain --daily                # day-by-day breakdown
clov gain --weekly               # week-by-week
clov gain --monthly              # month-by-month
clov gain --all                  # all breakdowns
clov gain --all --format json    # JSON export
clov gain --all --format csv     # CSV export
```

Example `clov gain` output:

```
╔══════════════════════════════════════════════════════╗
║           CLOV Token Savings (Global Scope)          ║
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

### Discover

Scans your Claude Code session history and shows where you wasted tokens running commands raw when clov could have handled them:

```bash
clov discover                    # current project, last 30 days
clov discover --all              # all Claude Code projects
clov discover --all --since 7    # last 7 days across all projects
clov discover -p myproject       # filter by project name
clov discover --format json
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
╚══════════════════════════════════════════════════════════╝
```

### Learn

Scans Claude Code error history and extracts CLI correction patterns — commands that failed and were retried with fixes. Useful for building project-level rules:

```bash
clov learn                       # current project, last 30 days
clov learn --all                 # all projects
clov learn --write-rules         # generate .claude/rules/cli-corrections.md
clov learn --min-confidence 0.8  # higher confidence threshold
```

### Misc

```bash
clov proxy git log --oneline -20 # bypass filtering, still tracks usage
clov verify                      # check hook integrity (SHA-256)
clov hook-audit --since 7        # hook rewrite metrics (needs CLOV_HOOK_AUDIT=1)
clov cc-economics                # Claude spend vs clov savings side-by-side
clov config                      # show current config
clov config --create             # generate ~/.config/clov/config.toml
clov init --show                 # show hook status and settings.json registration
```

---

## Auto-Rewrite Hook

CLAUDE.md instructions get ignored by subagents. The hook doesn't. It intercepts Bash commands at the shell level before execution, so clov runs regardless of whether the agent read your instructions.

```
Claude issues: "git status"
       |
       v
clov-rewrite.sh  ->  "clov git status"  (silent, before shell)
       |
       v
clov filters output, Claude gets: "3 modified, 1 untracked ✓"
```

**Coverage**: 100% across all subagents and conversations. Zero token overhead in context.

### Commands Rewritten

```
git           -> clov git ...
gh            -> clov gh ...
cargo         -> clov cargo ...
cat           -> clov read ...
rg / grep     -> clov grep ...
ls            -> clov ls
gt            -> clov gt ...
vitest        -> clov vitest run
tsc           -> clov tsc
eslint        -> clov lint
prettier      -> clov prettier
playwright    -> clov playwright
prisma        -> clov prisma
ruff          -> clov ruff ...
pytest        -> clov pytest
pip           -> clov pip ...
go            -> clov go ...
golangci-lint -> clov golangci-lint run
docker        -> clov docker ...
kubectl       -> clov kubectl ...
curl          -> clov curl
pnpm          -> clov pnpm ...
```

Commands already using `clov`, heredocs, and unrecognized commands pass through unchanged.

### Quick Install

```bash
clov init -g
# Installs hook to ~/.claude/hooks/clov-rewrite.sh
# Creates ~/.claude/CLOV.md (10 lines)
# Adds @CLOV.md reference to ~/.claude/CLAUDE.md
# Prompts to patch settings.json [y/N] — say yes

# Verify
clov init --show
```

Settings.json patching options:

```bash
clov init -g                 # default: prompts [y/N]
clov init -g --auto-patch    # patch immediately, no prompt
clov init -g --no-patch      # skip patching, print manual snippet
```

**Restart Claude Code after install.** Without the restart, the hook is registered but not active.

### Manual Install

If automatic patching fails:

```bash
# 1. Install hook file
clov init -g --no-patch  # prints JSON snippet

# 2. Add snippet to ~/.claude/settings.json manually

# 3. Restart Claude Code
```

Full manual:

```bash
mkdir -p ~/.claude/hooks
cp .claude/hooks/clov-rewrite.sh ~/.claude/hooks/clov-rewrite.sh
chmod +x ~/.claude/hooks/clov-rewrite.sh
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
            "command": "~/.claude/hooks/clov-rewrite.sh"
          }
        ]
      }
    ]
  }
}
```

### Suggest Hook (Audit Mode)

Prefer visibility over silent rewrites? The suggest hook detects clov-compatible commands and emits a system reminder instead of rewriting:

|          | Auto-Rewrite                      | Suggest                   |
| -------- | --------------------------------- | ------------------------- |
| Strategy | Modifies command before execution | Emits reminder to Claude  |
| Coverage | 100% forced                       | ~70-85% (model-dependent) |
| Use case | Production, guaranteed savings    | Auditing, learning mode   |

```bash
cp .claude/hooks/clov-suggest.sh ~/.claude/hooks/clov-suggest.sh
chmod +x ~/.claude/hooks/clov-suggest.sh
```

Register in settings.json the same way as the rewrite hook.

---

## Configuration

### Install Modes

| Command                    | Scope  | Hook | CLOV.md        | Context tokens | Use when       |
| -------------------------- | ------ | ---- | -------------- | -------------- | -------------- |
| `clov init -g`             | Global | yes  | yes (10 lines) | ~10            | Recommended    |
| `clov init -g --claude-md` | Global | no   | no             | ~2,000         | Legacy compat  |
| `clov init -g --hook-only` | Global | yes  | no             | 0              | Minimal        |
| `clov init`                | Local  | no   | no             | ~2,000         | Single project |

### Database

Token tracking lives in `~/.local/share/clov/history.db` by default.

Override with env var:

```bash
export CLOV_DB_PATH="/path/to/custom.db"
```

Or in `~/.config/clov/config.toml`:

```toml
[tracking]
database_path = "/path/to/custom.db"
```

Priority: env var > config file > default.

### Tee (Full Output Recovery)

When clov filters a failing command, the LLM loses raw details and often re-runs the same command multiple times. Tee saves unfiltered output and prints a one-line hint so the agent reads it directly:

```
✓ cargo test: 15 passed (1 suite, 0.01s)
[full output: ~/.local/share/clov/tee/1707753600_cargo_test.log]
```

Config (`~/.config/clov/config.toml`):

```toml
[tee]
enabled = true
mode = "failures"       # "failures" | "always" | "never"
max_files = 20
max_file_size = 1048576  # 1MB
```

Env overrides:

- `CLOV_TEE=0` — disable entirely
- `CLOV_TEE_DIR=/path` — custom output directory

---

## Troubleshooting

### settings.json patch failed

```bash
cat ~/.claude/settings.json | python3 -m json.tool  # validate JSON
clov init -g --no-patch                              # get snippet, patch manually
cp ~/.claude/settings.json.bak ~/.claude/settings.json  # restore backup
```

### Hook not running after install

Nine times out of ten: Claude Code was not restarted.

```bash
clov init --show                         # verify hook registration
cat ~/.claude/settings.json | grep clov-rewrite
# then restart Claude Code
```

### Leftover files after uninstall

```bash
clov init -g --uninstall    # removes hook, CLOV.md, settings.json entry
# or manually:
rm ~/.claude/hooks/clov-rewrite.sh
rm ~/.claude/CLOV.md
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

---

## Uninstall

```bash
clov init -g --uninstall

# Binary removal:
cargo uninstall clov      # if installed via cargo
brew uninstall clov       # macOS Homebrew
sudo apt remove clov      # Debian/Ubuntu
sudo dnf remove clov      # Fedora/RHEL
```

---

## Docs

- [INSTALL.md](INSTALL.md) — detailed installation and PATH verification
- [ARCHITECTURE.md](ARCHITECTURE.md) — module layout, filter strategies, performance targets
- [CLAUDE.md](CLAUDE.md) — Claude Code integration context
- [docs/AUDIT_GUIDE.md](docs/AUDIT_GUIDE.md) — analytics, export formats, quota estimation

---

## License

MIT — see [LICENSE](LICENSE).

---

<p align="center">
  <img src="https://skillicons.dev/icons?i=rust,python,go" height="24" />
  <br/><br/>
  <sub>maintained by <a href="https://github.com/alexandephilia">@alexandephilia</a> × claude</sub>
</p>
