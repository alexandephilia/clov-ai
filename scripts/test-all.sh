#!/usr/bin/env bash
#
# CLOV Smoke Test Suite
# Exercises every command to catch regressions after merge.
# Exit code: number of failures (0 = all green)
#
set -euo pipefail

PASS=0
FAIL=0
SKIP=0
FAILURES=()

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ── Helpers ──────────────────────────────────────────

assert_ok() {
    local name="$1"
    shift
    local output
    if output=$("$@" 2>&1); then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        cmd: %s\n" "$*"
        printf "        out: %s\n" "$(echo "$output" | head -3)"
    fi
}

assert_contains() {
    local name="$1"
    local needle="$2"
    shift 2
    local output
    if output=$("$@" 2>&1) && echo "$output" | grep -q "$needle"; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        expected: '%s'\n" "$needle"
        printf "        got: %s\n" "$(echo "$output" | head -3)"
    fi
}

assert_exit_ok() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        cmd: %s\n" "$*"
    fi
}

assert_help() {
    local name="$1"
    shift
    assert_contains "$name --help" "Usage:" "$@" --help
}

skip_test() {
    local name="$1"
    local reason="$2"
    SKIP=$((SKIP + 1))
    printf "  ${YELLOW}SKIP${NC}  %s (%s)\n" "$name" "$reason"
}

section() {
    printf "\n${BOLD}${CYAN}── %s ──${NC}\n" "$1"
}

# ── Preamble ─────────────────────────────────────────

CLOV=$(command -v clov || echo "")
if [[ -z "$CLOV" ]]; then
    echo "clov not found in PATH. Run: cargo install --path ."
    exit 1
fi

printf "${BOLD}CLOV Smoke Test Suite${NC}\n"
printf "Binary: %s\n" "$CLOV"
printf "Version: %s\n" "$(clov --version)"
printf "Date: %s\n" "$(date '+%Y-%m-%d %H:%M')"

# Need a git repo to test git commands
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Must run from inside a git repository."
    exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel)

# ── 1. Version & Help ───────────────────────────────

section "Version & Help"

assert_contains "clov --version" "clov" clov --version
assert_contains "clov --help" "Usage:" clov --help

# ── 2. Ls ────────────────────────────────────────────

section "Ls"

assert_ok      "clov files ."                     clov files .
assert_ok      "clov files -la ."                 clov files -la .
assert_ok      "clov files -lh ."                 clov files -lh .
assert_ok      "clov files -l src/"               clov files -l src/
assert_ok      "clov files src/ -l (flag after)"  clov files src/ -l
assert_ok      "clov files multi paths"           clov files src/ scripts/
assert_contains "clov files -a shows hidden"      ".git" clov files -a .
assert_contains "clov files shows sizes"          "K"  clov files src/
assert_contains "clov files shows dirs with /"    "/" clov files .

# ── 2b. Tree ─────────────────────────────────────────

section "Tree"

if command -v tree >/dev/null 2>&1; then
    assert_ok      "clov map ."                clov map .
    assert_ok      "clov map -L 2 ."           clov map -L 2 .
    assert_ok      "clov map -d -L 1 ."        clov map -d -L 1 .
    assert_contains "clov map shows src/"      "src" clov map -L 1 .
else
    skip_test "clov map" "tree not installed"
fi

# ── 3. Read ──────────────────────────────────────────

section "Read"

assert_ok      "clov view Cargo.toml"          clov view Cargo.toml
assert_ok      "clov view --level none Cargo.toml"  clov view --level none Cargo.toml
assert_ok      "clov view --level aggressive Cargo.toml" clov view --level aggressive Cargo.toml
assert_ok      "clov view -n Cargo.toml"       clov view -n Cargo.toml
assert_ok      "clov view --max-lines 5 Cargo.toml" clov view --max-lines 5 Cargo.toml

section "Read (stdin support)"

assert_ok      "clov view stdin pipe"          bash -c 'echo "fn main() {}" | clov view -'

# ── 4. Git ───────────────────────────────────────────

section "Git (existing)"

assert_ok      "clov git status"               clov git status
assert_ok      "clov git status --short"       clov git status --short
assert_ok      "clov git status -s"            clov git status -s
assert_ok      "clov git status --porcelain"   clov git status --porcelain
assert_ok      "clov git log"                  clov git log
assert_ok      "clov git log -5"               clov git log -- -5
assert_ok      "clov git diff"                 clov git diff
assert_ok      "clov git diff --stat"          clov git diff --stat

section "Git (new: branch, fetch, stash, worktree)"

assert_ok      "clov git branch"               clov git branch
assert_ok      "clov git fetch"                clov git fetch
assert_ok      "clov git stash list"           clov git stash list
assert_ok      "clov git worktree"             clov git worktree

section "Git (passthrough: unsupported subcommands)"

assert_ok      "clov git tag --list"           clov git tag --list
assert_ok      "clov git remote -v"            clov git remote -v
assert_ok      "clov git rev-parse HEAD"       clov git rev-parse HEAD

# ── 5. GitHub CLI ────────────────────────────────────

section "GitHub CLI"

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    assert_ok      "clov gh pr list"           clov gh pr list
    assert_ok      "clov gh run list"          clov gh run list
    assert_ok      "clov gh issue list"        clov gh issue list
    # pr create/merge/diff/comment/edit are write ops, test help only
    assert_help    "clov gh"                   clov gh
else
    skip_test "gh commands" "gh not authenticated"
fi

# ── 6. Cargo ─────────────────────────────────────────

section "Cargo (new)"

assert_ok      "clov cargo build"              clov cargo build
assert_ok      "clov cargo clippy"             clov cargo clippy
# cargo test exits non-zero due to pre-existing failures; check output ignoring exit code
output_cargo_test=$(clov cargo test 2>&1 || true)
if echo "$output_cargo_test" | grep -q "FAILURES\|test result:\|passed"; then
    PASS=$((PASS + 1))
    printf "  ${GREEN}PASS${NC}  %s\n" "clov cargo test"
else
    FAIL=$((FAIL + 1))
    FAILURES+=("clov cargo test")
    printf "  ${RED}FAIL${NC}  %s\n" "clov cargo test"
    printf "        got: %s\n" "$(echo "$output_cargo_test" | head -3)"
fi
assert_help    "clov cargo"                    clov cargo

# ── 7. Curl ──────────────────────────────────────────

section "Curl (new)"

assert_contains "clov curl JSON detect" "string" clov curl https://httpbin.org/json
assert_ok       "clov curl plain text"          clov curl https://httpbin.org/robots.txt
assert_help     "clov curl"                     clov curl

# ── 8. Npm / Npx ────────────────────────────────────

section "Npm / Npx (new)"

assert_help    "clov npm"                      clov npm
assert_help    "clov npx"                      clov npx

# ── 9. Pnpm ─────────────────────────────────────────

section "Pnpm"

assert_help    "clov pnpm"                     clov pnpm
assert_help    "clov pnpm build"               clov pnpm build
assert_help    "clov pnpm typecheck"           clov pnpm typecheck

if command -v pnpm >/dev/null 2>&1; then
    assert_ok  "clov pnpm help"                clov pnpm help
fi

# ── 10. Grep ─────────────────────────────────────────

section "Search"

assert_ok      "clov search pattern"             clov search "pub fn" src/
assert_contains "clov search finds results"      "pub fn" clov search "pub fn" src/
assert_ok      "clov search with file type"      clov search "pub fn" src/ -t rust

section "Search (extra args passthrough)"

assert_ok      "clov search -i case insensitive" clov search "fn" src/ -i
assert_ok      "clov search -A context lines"    clov search "fn run" src/ -A 2

# ── 11. Find ─────────────────────────────────────────

section "Scan"

assert_ok      "clov scan *.rs"                clov scan "*.rs" src/
assert_contains "clov scan shows files"        ".rs" clov scan "*.rs" src/

# ── 12. Json ─────────────────────────────────────────

section "Schema"

# Create temp JSON file for testing
TMPJSON=$(mktemp /tmp/clov-test-XXXXX.json)
echo '{"name":"test","count":42,"items":[1,2,3]}' > "$TMPJSON"

assert_ok      "clov schema file"                clov schema "$TMPJSON"
assert_contains "clov schema shows schema"       "string" clov schema "$TMPJSON"

rm -f "$TMPJSON"

# ── 13. Deps ─────────────────────────────────────────

section "Graph"

assert_ok      "clov graph ."                   clov graph .
assert_contains "clov graph shows Cargo"        "Cargo" clov graph .

# ── 14. Env ──────────────────────────────────────────

section "Vars"

assert_ok      "clov vars"                      clov vars
assert_ok      "clov vars --filter PATH"        clov vars --filter PATH

# ── 16. Log ──────────────────────────────────────────

section "Logs"

TMPLOG=$(mktemp /tmp/clov-log-XXXXX.log)
for i in $(seq 1 20); do
    echo "[2025-01-01 12:00:00] INFO: repeated message" >> "$TMPLOG"
done
echo "[2025-01-01 12:00:01] ERROR: something failed" >> "$TMPLOG"

assert_ok      "clov logs file"                 clov logs "$TMPLOG"

rm -f "$TMPLOG"

# ── 17. Summary ──────────────────────────────────────

section "Digest"

assert_ok      "clov digest echo hello"       clov digest echo hello

# ── 18. Err ──────────────────────────────────────────

section "Fail"

assert_ok      "clov fail echo ok"              clov fail echo ok

# ── 19. Test runner ──────────────────────────────────

section "Check runner"

assert_ok      "clov check echo ok"             clov check echo ok

# ── 20. Gain ─────────────────────────────────────────

section "Pulse"

assert_ok      "clov pulse"                     clov pulse
assert_ok      "clov pulse --history"           clov pulse --history

# ── 21. Config & Init ────────────────────────────────

section "Settings & Hook"

assert_ok      "clov settings"                   clov settings
assert_ok      "clov hook --show"                clov hook --show

# ── 22. Wget ─────────────────────────────────────────

section "Wget"

if command -v wget >/dev/null 2>&1; then
    assert_ok  "clov fetch stdout"              clov fetch https://httpbin.org/robots.txt -O
else
    skip_test "clov fetch" "wget not installed"
fi

# ── 23. Tsc / Lint / Prettier / Next / Playwright ───

section "JS Tooling (help only, no project context)"

assert_help    "clov tsc"                      clov tsc
assert_help    "clov lint"                     clov lint
assert_help    "clov prettier"                 clov prettier
assert_help    "clov next"                     clov next
assert_help    "clov playwright"               clov playwright

# ── 24. Prisma ───────────────────────────────────────

section "Prisma (help only)"

assert_help    "clov prisma"                   clov prisma

# ── 25. Vitest ───────────────────────────────────────

section "Vitest (help only)"

assert_help    "clov vitest"                   clov vitest

# ── 26. Docker / Kubectl (help only) ────────────────

section "Docker / Kubectl (help only)"

assert_help    "clov docker"                   clov docker
assert_help    "clov kubectl"                  clov kubectl

# ── 27. Python (conditional) ────────────────────────

section "Python (conditional)"

if command -v pytest &>/dev/null; then
    assert_help    "clov pytest"                    clov pytest --help
else
    skip_test "clov pytest" "pytest not installed"
fi

if command -v ruff &>/dev/null; then
    assert_help    "clov ruff"                      clov ruff --help
else
    skip_test "clov ruff" "ruff not installed"
fi

if command -v pip &>/dev/null; then
    assert_help    "clov pip"                       clov pip --help
else
    skip_test "clov pip" "pip not installed"
fi

# ── 28. Go (conditional) ────────────────────────────

section "Go (conditional)"

if command -v go &>/dev/null; then
    assert_help    "clov go"                        clov go --help
    assert_help    "clov go test"                   clov go test -h
    assert_help    "clov go build"                  clov go build -h
    assert_help    "clov go vet"                    clov go vet -h
else
    skip_test "clov go" "go not installed"
fi

if command -v golangci-lint &>/dev/null; then
    assert_help    "clov golangci-lint"             clov golangci-lint --help
else
    skip_test "clov golangci-lint" "golangci-lint not installed"
fi

# ── 29. Global flags ────────────────────────────────

section "Global flags"

assert_ok      "clov -u files ."               clov -u files .
assert_ok      "clov --skip-env npm --help"    clov --skip-env npm --help

# ── 30. CcEconomics ─────────────────────────────────

section "CcEconomics"

assert_ok      "clov cc-economics"             clov cc-economics

# ── 31. Learn ───────────────────────────────────────

section "Learn"

assert_ok      "clov adapt --help"             clov adapt --help
assert_ok      "clov adapt (no sessions)"      clov adapt --since 0 2>&1 || true

# ══════════════════════════════════════════════════════
# Report
# ══════════════════════════════════════════════════════

printf "\n${BOLD}══════════════════════════════════════${NC}\n"
printf "${BOLD}Results: ${GREEN}%d passed${NC}, ${RED}%d failed${NC}, ${YELLOW}%d skipped${NC}\n" "$PASS" "$FAIL" "$SKIP"

if [[ ${#FAILURES[@]} -gt 0 ]]; then
    printf "\n${RED}Failures:${NC}\n"
    for f in "${FAILURES[@]}"; do
        printf "  - %s\n" "$f"
    done
fi

printf "${BOLD}══════════════════════════════════════${NC}\n"

exit "$FAIL"
