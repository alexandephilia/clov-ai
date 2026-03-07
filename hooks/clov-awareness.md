# CLOV - Token-Optimized CLI Proxy

## Core Concept

**What**: Shell filter that intercepts dev commands and compresses output before it reaches the LLM
**Why**: Reduce token consumption by 60-90% without losing critical information
**How**: Smart filtering (strips ANSI/progress bars), grouping (errors by file), truncation, deduplication

**Example transformation**:

```
Raw git status:        ~300 tokens → "On branch main\nYour branch is up to date...\n\nChanges not staged..."
Filtered (clov):       ~60 tokens  → "3 modified, 1 untracked ✓"
```

## Decision Tree: When to Use What

### 1. Meta Commands → Always Use `clov` Directly

These analyze CLOV's own behavior. Hook doesn't rewrite them.

```bash
clov gain              # Token savings analytics
clov gain --history    # Per-command breakdown
clov cc-savings        # Claude spend vs CLOV savings
clov discover          # Find missed optimization opportunities
clov proxy <cmd>       # Bypass filtering (debugging/full output)
```

### 2. Dev Commands → Let Hook Auto-Rewrite

**Default behavior**: Type raw commands, hook rewrites transparently.

```
You type:       git status
Hook rewrites:  clov git status  (automatic, <1ms overhead)
You see:        Filtered output
```

**Supported commands** (auto-rewritten):

- `git`, `gh`, `gt` (Graphite)
- `cargo`, `go`, `npm`, `pnpm`, `pip`
- `tsc`, `eslint`, `prettier`, `next`, `vitest`, `playwright`, `prisma`
- `ruff`, `pytest`, `mypy`, `golangci-lint`
- `docker`, `kubectl`, `curl`
- `cat` → `clov read`, `grep`/`rg` → `clov grep`, `ls` → `clov ls`

**Coverage**: 100% (works across all subagents, doesn't rely on CLAUDE.md being read)

### 3. Need Full Output? → Use `clov proxy`

```bash
clov proxy git log --oneline -20   # Bypass filtering, still track metrics
```

**When to use**:

- Debugging CLOV filter behavior
- Suspect filtering hides needed info
- Compare filtered vs raw output
- Workaround for filter bugs

### 4. MCP Tools → Route via `clov mcp proxy`

For MCP servers like Exa, route the entire server through CLOV in your Claude Code `settings.json`.

```json
{
  "mcpServers": {
    "exa": {
      "command": "clov",
      "args": ["mcp", "proxy", "npx", "-y", "exa-mcp-server"],
      "env": { "EXA_API_KEY": "..." }
    }
  }
}
```

**Supported MCP Filters**:

- `exa` (web_search, crawling) → Strips nav chrome, cookie notices, footers. 85-95% savings.

## Hook Mechanism

```
WITHOUT HOOK:
Claude → "git status" → shell → verbose output (300 tokens) → context

WITH HOOK:
Claude → "git status" → hook intercepts → "clov git status" → filtered (60 tokens) → context
```

**Key properties**:

- Runs at shell level (before execution)
- Transparent to user (no manual prefixing)
- Minimal overhead (<1ms startup time)
- Graceful fallback (if filter fails, runs raw command)
- Works even if subagents ignore CLAUDE.md

## Installation Check

```bash
clov --version         # Should show: clov 0.26.2+
clov gain              # Should print stats table (not "command not found")
which clov             # Verify binary path
clov init --show       # Verify hook registered in settings.json
```

**If any fail**: CLOV not installed or not in PATH.

## Token Savings (Real Data)

| Command Type     | Savings | Strategy                        |
| ---------------- | ------- | ------------------------------- |
| Git operations   | 70-85%  | Compact diffs, stat summaries   |
| Test runners     | 90-99%  | Failures only, grouped          |
| Linters          | 80-85%  | Group by rule/file              |
| Package managers | 70-90%  | Compact dependency trees        |
| Build tools      | 85-90%  | Errors/warnings only            |
| MCP Tool Results | 85-95%  | JSON-aware web chrome stripping |

**Overall**: 60-95% reduction across all dev operations and tool use.

**Note**: Percentages vary by output size. Small outputs show lower % but still reduce absolute tokens.

## Fallback Behavior

CLOV never breaks your workflow:

- **Unrecognized command**: Passes through unchanged
- **Filter fails**: Executes raw command, logs error
- **Exit codes**: Always preserved (CI/CD compatible)

## Reference Docs

- **CLAUDE.md**: Full command reference, architecture, dev guidelines
- **README.md**: User-facing features, installation, examples
- **ARCHITECTURE.md**: Filter strategies, performance targets, module design
