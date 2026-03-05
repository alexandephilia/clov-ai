# CLOV Installation Guide - For AI Coding Assistants

## ⚠️ Name Collision Warning

**There are TWO completely different projects named "clov":**

1. ✅ **Clov Token Omitter** (this project) - LLM token optimizer
   - Repos: `alexandephilia/clov-ai`
   - Has `clov gain` command for token savings stats

2. ❌ **Rust Type Kit** (reachingforthejack/clov) - DIFFERENT PROJECT
   - Rust codebase query tool and type generator
   - **DO NOT install if you want token optimization**

## Pre-Installation Check (REQUIRED)

**AI assistants should ALWAYS verify if CLOV is already installed before attempting installation.**

```bash
# Check if CLOV is installed
clov --version

# CRITICAL: Verify it's the Token Omitter (not Type Kit)
clov gain    # Should show token savings stats, NOT "command not found"

# Check installation path
which clov
```

If `clov gain` works, you have the **correct** CLOV installed. **DO NOT reinstall**. Skip to "Project Initialization".

If `clov gain` fails but `clov --version` succeeds, you have the **wrong** CLOV (Type Kit). Uninstall and reinstall the correct one (see below).

## Installation (only if CLOV not available or wrong CLOV installed)

### Step 0: Uninstall Wrong CLOV (if needed)

If you accidentally installed Rust Type Kit:

```bash
cargo uninstall clov
```

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/alexandephilia/clov-ai/master/install.sh | sh
```

After installation, **verify you have the correct clov**:
```bash
clov gain  # Must show token savings stats (not "command not found")
```

### Alternative: Manual Installation

```bash
# From clov-ai repository (NOT reachingforthejack!)
cargo install --git https://github.com/alexandephilia/clov-ai

# OR (if published and correct on crates.io)
cargo install clov

# ALWAYS VERIFY after installation
clov gain  # MUST show token savings, not "command not found"
```

⚠️ **WARNING**: `cargo install clov` from crates.io might install the wrong package. Always verify with `clov gain`.

## Project Initialization

### Which mode to choose?

```
  Do you want CLOV active across ALL Claude Code projects?
  │
  ├─ YES → clov init -g              (recommended)
  │         Hook + CLOV.md (~10 tokens in context)
  │         Commands auto-rewritten transparently
  │
  ├─ YES, minimal → clov init -g --hook-only
  │         Hook only, nothing added to CLAUDE.md
  │         Zero tokens in context
  │
  └─ NO, single project → clov init
            Local CLAUDE.md only (137 lines)
            No hook, no global effect
```

### Recommended: Global Hook-First Setup

**Best for: All projects, automatic CLOV usage**

```bash
clov init -g
# → Installs hook to ~/.claude/hooks/clov-rewrite.sh
# → Creates ~/.claude/CLOV.md (10 lines, meta commands only)
# → Adds @CLOV.md reference to ~/.claude/CLAUDE.md
# → Prompts: "Patch settings.json? [y/N]"
# → If yes: patches + creates backup (~/.claude/settings.json.bak)

# Automated alternatives:
clov init -g --auto-patch    # Patch without prompting
clov init -g --no-patch      # Print manual instructions instead

# Verify installation
clov init --show  # Check hook is installed and executable
```

**Token savings**: ~99.5% reduction (2000 tokens → 10 tokens in context)

**What is settings.json?**
Claude Code's hook registry. CLOV adds a PreToolUse hook that rewrites commands transparently. Without this, Claude won't invoke the hook automatically.

```
  Claude Code          settings.json        clov-rewrite.sh        CLOV binary
       │                    │                     │                    │
       │  "git status"      │                     │                    │
       │ ──────────────────►│                     │                    │
       │                    │  PreToolUse trigger  │                    │
       │                    │ ───────────────────►│                    │
       │                    │                     │  rewrite command   │
       │                    │                     │  → clov git status  │
       │                    │◄────────────────────│                    │
       │                    │  updated command     │                    │
       │                    │                                          │
       │  execute: clov git status                                      │
       │ ─────────────────────────────────────────────────────────────►│
       │                                                               │  filter
       │  "3 modified, 1 untracked ✓"                                  │
       │◄──────────────────────────────────────────────────────────────│
```

**Backup Safety**:
CLOV backs up existing settings.json before changes. Restore if needed:
```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

### Alternative: Local Project Setup

**Best for: Single project without hook**

```bash
cd /path/to/your/project
clov init  # Creates ./CLAUDE.md with full CLOV instructions (137 lines)
```

**Token savings**: Instructions loaded only for this project

### Upgrading from Previous Version

#### From old 137-line CLAUDE.md injection (pre-0.22)

```bash
clov init -g  # Automatically migrates to hook-first mode
# → Removes old 137-line block
# → Installs hook + CLOV.md
# → Adds @CLOV.md reference
```

#### From old hook with inline logic (pre-0.24) — ⚠️ Breaking Change

CLOV 0.24.0 replaced the inline command-detection hook (~200 lines) with a **thin delegator** that calls `clov rewrite`. The binary now contains the rewrite logic, so adding new commands no longer requires a hook update.

The old hook still works but won't benefit from new rules added in future releases.

```bash
# Upgrade hook to thin delegator
clov init --global

# Verify the new hook is active
clov init --show
# Should show: ✅ Hook: ... (thin delegator, up to date)
```

## Common User Flows

### First-Time User (Recommended)
```bash
# 1. Install CLOV
cargo install --git https://github.com/alexandephilia/clov-ai
clov gain  # Verify (must show token stats)

# 2. Setup with prompts
clov init -g
# → Answer 'y' when prompted to patch settings.json
# → Creates backup automatically

# 3. Restart Claude Code
# 4. Test: git status (should use clov)
```

### CI/CD or Automation
```bash
# Non-interactive setup (no prompts)
clov init -g --auto-patch

# Verify in scripts
clov init --show | grep "Hook:"
```

### Conservative User (Manual Control)
```bash
# Get manual instructions without patching
clov init -g --no-patch

# Review printed JSON snippet
# Manually edit ~/.claude/settings.json
# Restart Claude Code
```

### Temporary Trial
```bash
# Install hook
clov init -g --auto-patch

# Later: remove everything
clov init -g --uninstall

# Restore backup if needed
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

## Installation Verification

```bash
# Basic test
clov ls .

# Test with git
clov git status

# Test with pnpm (fork only)
clov pnpm list

# Test with Vitest (feat/vitest-support branch only)
clov vitest run
```

## Uninstalling

### Complete Removal (Global Installations Only)

```bash
# Complete removal (global installations only)
clov init -g --uninstall

# What gets removed:
#   - Hook: ~/.claude/hooks/clov-rewrite.sh
#   - Context: ~/.claude/CLOV.md
#   - Reference: @CLOV.md line from ~/.claude/CLAUDE.md
#   - Registration: CLOV hook entry from settings.json

# Restart Claude Code after uninstall
```

**For Local Projects**: Manually remove CLOV block from `./CLAUDE.md`

### Binary Removal

```bash
# If installed via cargo
cargo uninstall clov

# If installed via package manager
brew uninstall clov          # macOS Homebrew
sudo apt remove clov         # Debian/Ubuntu
sudo dnf remove clov         # Fedora/RHEL
```

### Restore from Backup (if needed)

```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

## Essential Commands

### Files
```bash
clov ls .              # Compact tree view
clov read file.rs      # Optimized reading
clov grep "pattern" .  # Grouped search results
```

### Git
```bash
clov git status        # Compact status
clov git log -n 10     # Condensed logs
clov git diff          # Optimized diff
clov git add .         # → "ok ✓"
clov git commit -m "msg"  # → "ok ✓ abc1234"
clov git push          # → "ok ✓ main"
```

### Pnpm (fork only)
```bash
clov pnpm list         # Dependency tree (-70% tokens)
clov pnpm outdated     # Available updates (-80-90%)
clov pnpm install pkg  # Silent installation
```

### Tests
```bash
clov test cargo test   # Failures only (-90%)
clov vitest run        # Filtered Vitest output (-99.6%)
```

### Statistics
```bash
clov gain              # Token savings
clov gain --graph      # With ASCII graph
clov gain --history    # With command history
```

## Validated Token Savings

### Production T3 Stack Project
| Operation | Standard | CLOV | Reduction |
|-----------|----------|-----|-----------|
| `vitest run` | 102,199 chars | 377 chars | **-99.6%** |
| `git status` | 529 chars | 217 chars | **-59%** |
| `pnpm list` | ~8,000 tokens | ~2,400 | **-70%** |
| `pnpm outdated` | ~12,000 tokens | ~1,200-2,400 | **-80-90%** |

### Typical Claude Code Session (30 min)
- **Without CLOV**: ~150,000 tokens
- **With CLOV**: ~45,000 tokens
- **Savings**: **70% reduction**

## Troubleshooting

### CLOV command not found after installation
```bash
# Check PATH
echo $PATH | grep -o '[^:]*\.cargo[^:]*'

# Add to PATH if needed (~/.bashrc or ~/.zshrc)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc
```

### CLOV command not available (e.g., vitest)
```bash
# Check branch
cd /path/to/clov
git branch

# Switch to feat/vitest-support if needed
git checkout feat/vitest-support

# Reinstall
cargo install --path . --force
```

### Compilation error
```bash
# Update Rust
rustup update stable

# Clean and recompile
cargo clean
cargo build --release
cargo install --path . --force
```

## Support and Contributing

- **Website**: https://github.com/alexandephilia/clov-ai
- **Contact**: https://github.com/alexandephilia/clov-ai/issues
- **Troubleshooting**: See [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) for common issues
- **GitHub issues**: https://github.com/alexandephilia/clov-ai/issues
- **Pull Requests**: https://github.com/alexandephilia/clov-ai/pulls

⚠️ **If you installed the wrong clov (Type Kit)**, see [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md#problem-clov-gain-command-not-found)

## AI Assistant Checklist

Before each session:

- [ ] Verify CLOV is installed: `clov --version`
- [ ] If not installed → follow "Install from fork"
- [ ] If project not initialized → `clov init`
- [ ] Use `clov` for ALL git/pnpm/test/vitest commands
- [ ] Check savings: `clov gain`

**Golden Rule**: AI coding assistants should ALWAYS use `clov` as a proxy for shell commands that generate verbose output (git, pnpm, npm, cargo test, vitest, docker, kubectl).
