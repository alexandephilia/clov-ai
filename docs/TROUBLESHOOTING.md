# CLOV Troubleshooting Guide

## Problem: "clov gain" command not found

### Symptom
```bash
$ clov --version
clov 1.0.0  # (or similar)

$ clov gain
clov: 'gain' is not a clov command. See 'clov --help'.
```

### Root Cause
You installed the **wrong clov package**. You have **Rust Type Kit** (reachingforthejack/clov) instead of **Clov Token Omitter** (alexandephilia/clov-ai).

### Solution

**1. Uninstall the wrong package:**
```bash
cargo uninstall clov
```

**2. Install the correct one (Token Omitter):**

#### Quick Install (Linux/macOS)
```bash
curl -fsSL https://github.com/alexandephilia/clov-ai/blob/master/install.sh | sh
```

#### Alternative: Manual Installation
```bash
cargo install --git https://github.com/alexandephilia/clov-ai
```

**3. Verify installation:**
```bash
clov --version
clov gain  # MUST show token savings stats, not error
```

If `clov gain` now works, installation is correct.

---

## Problem: Confusion Between Two "clov" Projects

### The Two Projects

| Project | Repository | Purpose | Key Command |
|---------|-----------|---------|-------------|
| **Clov Token Omitter** ✅ | alexandephilia/clov-ai | LLM token optimizer for Claude Code | `clov gain` |
| **Rust Type Kit** ❌ | reachingforthejack/clov | Rust codebase query and type generator | `clov query` |

### How to Identify Which One You Have

```bash
# Check if "gain" command exists
clov gain

# Token Omitter → Shows token savings stats
# Type Kit → Error: "gain is not a clov command"
```

---

## Problem: cargo install clov installs wrong package

### Why This Happens
If **Rust Type Kit** is published to crates.io under the name `clov`, running `cargo install clov` will install the wrong package.

### Solution
**NEVER use** `cargo install clov` without verifying.

**Always use explicit repository URLs:**

```bash
# CORRECT - Token Omitter
cargo install --git https://github.com/alexandephilia/clov-ai

# OR install from fork
git clone https://github.com/alexandephilia/clov-ai.git
cd clov && git checkout feat/all-features
cargo install --path . --force
```

**After any installation, ALWAYS verify:**
```bash
clov gain  # Must work if you want Token Omitter
```

---

## Problem: CLOV not working in Claude Code

### Symptom
Claude Code doesn't seem to be using clov, outputs are verbose.

### Checklist

**1. Verify clov is installed and correct:**
```bash
clov --version
clov gain  # Must show stats
```

**2. Initialize clov for Claude Code:**
```bash
# Global (all projects)
clov init --global

# Per-project
cd /your/project
clov init
```

**3. Verify CLAUDE.md file exists:**
```bash
# Check global
cat ~/.claude/CLAUDE.md | grep clov

# Check project
cat ./CLAUDE.md | grep clov
```

**4. Install auto-rewrite hook (recommended for automatic CLOV usage):**

**Option A: Automatic (recommended)**
```bash
clov init -g
# → Installs hook + CLOV.md automatically
# → Follow printed instructions to add hook to ~/.claude/settings.json
# → Restart Claude Code

# Verify installation
clov init --show  # Should show "✅ Hook: executable, with guards"
```

**Option B: Manual (fallback)**
```bash
# Copy hook to Claude Code hooks directory
mkdir -p ~/.claude/hooks
cp .claude/hooks/clov-rewrite.sh ~/.claude/hooks/
chmod +x ~/.claude/hooks/clov-rewrite.sh
```

Then add to `~/.claude/settings.json` (replace `~` with full path):
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/Users/yourname/.claude/hooks/clov-rewrite.sh"
          }
        ]
      }
    ]
  }
}
```

**Note**: Use absolute path in `settings.json`, not `~/.claude/...`

---

## Problem: "command not found: clov" after installation

### Symptom
```bash
$ cargo install --path . --force
   Compiling clov v0.7.1
    Finished release [optimized] target(s)
  Installing ~/.cargo/bin/clov

$ clov --version
zsh: command not found: clov
```

### Root Cause
`~/.cargo/bin` is not in your PATH.

### Solution

**1. Check if cargo bin is in PATH:**
```bash
echo $PATH | grep -o '[^:]*\.cargo[^:]*'
```

**2. If not found, add to PATH:**

For **bash** (`~/.bashrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For **zsh** (`~/.zshrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For **fish** (`~/.config/fish/config.fish`):
```fish
set -gx PATH $HOME/.cargo/bin $PATH
```

**3. Reload shell config:**
```bash
source ~/.bashrc  # or ~/.zshrc or restart terminal
```

**4. Verify:**
```bash
which clov
clov --version
clov gain
```

---

## Problem: Compilation errors during installation

### Symptom
```bash
$ cargo install --path .
error: failed to compile clov v0.7.1
```

### Solutions

**1. Update Rust toolchain:**
```bash
rustup update stable
rustup default stable
```

**2. Clean and rebuild:**
```bash
cargo clean
cargo build --release
cargo install --path . --force
```

**3. Check Rust version (minimum required):**
```bash
rustc --version  # Should be 1.70+ for most features
```

**4. If still fails, report issue:**
- GitHub: https://github.com/alexandephilia/clov-ai/issues

---

## Need More Help?

**Report issues:**
- Fork-specific: https://github.com/alexandephilia/clov-ai/issues
- Upstream: https://github.com/alexandephilia/clov-ai/issues

**Run the diagnostic script:**
```bash
# From the clov repository root
bash scripts/check-installation.sh
```

This script will check:
- ✅ CLOV installed and in PATH
- ✅ Correct version (Token Omitter, not Type Kit)
- ✅ Available features (pnpm, vitest, next, etc.)
- ✅ Claude Code integration (CLAUDE.md files)
- ✅ Auto-rewrite hook status

The script provides specific fix commands for any issues found.
