#!/usr/bin/env bash
#
# CLOV Smoke Tests — Aristote Project (Vite + React + TS + ESLint)
# Tests CLOV commands in a real JS/TS project context.
# Usage: bash scripts/test-aristote.sh
#
set -euo pipefail

ARISTOTE="/Users/florianbruniaux/Sites/MethodeAristote/aristote-school-boost"

PASS=0
FAIL=0
SKIP=0
FAILURES=()

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

assert_ok() {
    local name="$1"; shift
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
    local name="$1"; local needle="$2"; shift 2
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

# Allow non-zero exit but check output
assert_output() {
    local name="$1"; local needle="$2"; shift 2
    local output
    output=$("$@" 2>&1) || true
    if echo "$output" | grep -q "$needle"; then
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

skip_test() {
    local name="$1"; local reason="$2"
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

if [[ ! -d "$ARISTOTE" ]]; then
    echo "Aristote project not found at $ARISTOTE"
    exit 1
fi

printf "${BOLD}CLOV Smoke Tests — Aristote Project${NC}\n"
printf "Binary: %s (%s)\n" "$CLOV" "$(clov --version)"
printf "Project: %s\n" "$ARISTOTE"
printf "Date: %s\n\n" "$(date '+%Y-%m-%d %H:%M')"

# ── 1. File exploration ──────────────────────────────

section "Files & Scan"

assert_ok       "clov files project root"           clov files "$ARISTOTE"
assert_ok       "clov files src/"                   clov files "$ARISTOTE/src"
assert_ok       "clov files --depth 3"              clov files --depth 3 "$ARISTOTE/src"
assert_contains "clov files shows components/"      "components" clov files "$ARISTOTE/src"
assert_ok       "clov scan *.tsx"                   clov scan "*.tsx" "$ARISTOTE/src"
assert_ok       "clov scan *.ts"                    clov scan "*.ts" "$ARISTOTE/src"
assert_contains "clov scan finds App.tsx"           "App.tsx" clov scan "*.tsx" "$ARISTOTE/src"

# ── 2. Read ──────────────────────────────────────────

section "View"

assert_ok       "clov view tsconfig.json"        clov view "$ARISTOTE/tsconfig.json"
assert_ok       "clov view package.json"         clov view "$ARISTOTE/package.json"
assert_ok       "clov view App.tsx"              clov view "$ARISTOTE/src/App.tsx"
assert_ok       "clov view --level aggressive"   clov view --level aggressive "$ARISTOTE/src/App.tsx"
assert_ok       "clov view --max-lines 10"       clov view --max-lines 10 "$ARISTOTE/src/App.tsx"

# ── 3. Grep ──────────────────────────────────────────

section "Search"

assert_ok       "clov search import"               clov search "import" "$ARISTOTE/src"
assert_ok       "clov search with type filter"     clov search "useState" "$ARISTOTE/src" -t tsx
assert_contains "clov search finds components"     "import" clov search "import" "$ARISTOTE/src"

# ── 4. Git ───────────────────────────────────────────

section "Git (in Aristote repo)"

# clov git doesn't support -C, use git -C via subshell
assert_ok       "clov git status"                bash -c "cd $ARISTOTE && clov git status"
assert_ok       "clov git log"                   bash -c "cd $ARISTOTE && clov git log"
assert_ok       "clov git branch"                bash -c "cd $ARISTOTE && clov git branch"

# ── 5. Deps ──────────────────────────────────────────

section "Graph"

assert_ok       "clov graph"                     clov graph "$ARISTOTE"
assert_contains "clov graph shows package.json"  "package.json" clov graph "$ARISTOTE"

# ── 6. Json ──────────────────────────────────────────

section "Schema"

assert_ok       "clov schema tsconfig"           clov schema "$ARISTOTE/tsconfig.json"
assert_ok       "clov schema package.json"       clov schema "$ARISTOTE/package.json"

# ── 7. Env ───────────────────────────────────────────

section "Vars"

assert_ok       "clov vars"                      clov vars
assert_ok       "clov vars --filter NODE"        clov vars --filter NODE

# ── 8. Tsc ───────────────────────────────────────────

section "TypeScript (tsc)"

if command -v npx >/dev/null 2>&1 && [[ -d "$ARISTOTE/node_modules" ]]; then
    assert_output "clov tsc (in aristote)" "error\|✅\|TS" clov tsc --project "$ARISTOTE"
else
    skip_test "clov tsc" "node_modules not installed"
fi

# ── 9. ESLint ────────────────────────────────────────

section "ESLint (lint)"

if command -v npx >/dev/null 2>&1 && [[ -d "$ARISTOTE/node_modules" ]]; then
    assert_output "clov lint (in aristote)" "error\|warning\|✅\|violations\|clean" clov lint --project "$ARISTOTE"
else
    skip_test "clov lint" "node_modules not installed"
fi

# ── 10. Build (Vite) ─────────────────────────────────

section "Build (Vite via clov next)"

if [[ -d "$ARISTOTE/node_modules" ]]; then
    # Aristote uses Vite, not Next — but clov next wraps the build script
    # Test with a timeout since builds can be slow
    skip_test "clov next build" "Vite project, not Next.js — use npm run build directly"
else
    skip_test "clov next build" "node_modules not installed"
fi

# ── 11. Diff ─────────────────────────────────────────

section "Patch"

# Diff two config files that exist in the project
assert_ok       "clov patch tsconfigs"           clov patch "$ARISTOTE/tsconfig.json" "$ARISTOTE/tsconfig.app.json"

# ── 12. Summary & Err ────────────────────────────────

section "Digest & Fail"

assert_ok       "clov digest ls"                 clov digest ls "$ARISTOTE/src"
assert_ok       "clov fail ls"                   clov fail ls "$ARISTOTE/src"

# ── 13. Gain ─────────────────────────────────────────

section "Pulse (after above commands)"

assert_ok       "clov pulse"                     clov pulse
assert_ok       "clov pulse --history"           clov pulse --history

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
